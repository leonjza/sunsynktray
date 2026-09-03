use super::error::AuthenticationExpired;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

type ProgressCallback = Arc<dyn Fn(&str) + Send + Sync>;

pub(crate) struct SunsynkClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) access_expires_at: Option<Instant>,
    pub(crate) progress: Option<ProgressCallback>,
}

impl SunsynkClient {
    pub(crate) fn new(base_url: String, username: String, password: String) -> Result<Self> {
        Ok(Self {
            http: Client::builder()
                .timeout(Duration::from_secs(20))
                .build()
                .context("could not build SunSynk HTTP client")?,
            base_url: base_url.trim_end_matches('/').into(),
            username,
            password,
            access_token: None,
            refresh_token: None,
            access_expires_at: None,
            progress: None,
        })
    }

    pub(crate) fn with_refresh_token(mut self, token: Option<String>) -> Self {
        self.refresh_token = token;
        self
    }

    pub(crate) fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub(crate) fn with_progress<F>(mut self, progress: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.progress = Some(Arc::new(progress));
        self
    }

    pub(crate) fn report_progress(&self, message: &str) {
        if let Some(progress) = &self.progress {
            progress(message);
        }
    }

    /// Fetch a raw authenticated response for the opt-in API inspection tool.
    /// Normal application code should use the typed methods instead.
    pub(crate) async fn get(
        &mut self,
        path: &str,
        params: Option<&[(&str, String)]>,
    ) -> Result<Value> {
        if self.access_token.is_none()
            || self.access_expires_at.is_some_and(|expires| {
                expires.saturating_duration_since(Instant::now()) <= Duration::from_secs(30)
            })
        {
            self.authenticate().await?;
        }
        let query = params.map(|items| {
            items
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>()
        });
        let response = match self.request("GET", path, query.as_deref(), None).await {
            Err(error) if error.downcast_ref::<AuthenticationExpired>().is_some() => {
                self.access_token = None;
                self.authenticate()
                    .await
                    .with_context(|| format!("re-authenticating before GET {path}"))?;
                self.request("GET", path, query.as_deref(), None)
                    .await
                    .with_context(|| format!("retrying GET {path}"))?
            }
            result => result.with_context(|| format!("GET {path}"))?,
        };
        if response.get("success").and_then(Value::as_bool) != Some(true) {
            bail!(
                "GET {path}: {}",
                response
                    .get("msg")
                    .and_then(Value::as_str)
                    .unwrap_or("SunSynk API request failed")
            );
        }
        Ok(response)
    }

    pub(crate) async fn request(
        &self,
        method: &str,
        path: &str,
        params: Option<&[(&str, String)]>,
        json: Option<&Value>,
    ) -> Result<Value> {
        let mut request = self
            .http
            .request(method.parse()?, format!("{}{}", self.base_url, path))
            .header("Accept", "application/json");
        if let Some(token) = &self.access_token {
            request = request.bearer_auth(token);
        }
        if let Some(params) = params {
            request = request.query(params);
        }
        if let Some(json) = json {
            request = request.json(json);
        }
        let response = request
            .send()
            .await
            .with_context(|| format!("sending {method} {path}"))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!(AuthenticationExpired));
        }
        let response = response
            .error_for_status()
            .with_context(|| format!("SunSynk returned an HTTP error for {method} {path}"))?;
        let body: Value = response
            .json()
            .await
            .with_context(|| format!("decoding SunSynk response for {method} {path}"))?;
        Ok(body)
    }
}
