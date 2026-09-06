use super::error::AuthenticationExpired;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

type ProgressCallback = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone)]
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

    pub(crate) async fn ensure_authenticated(&mut self) -> Result<()> {
        if self.access_token.is_none()
            || self.access_expires_at.is_some_and(|expires| {
                expires.saturating_duration_since(Instant::now()) <= Duration::from_secs(30)
            })
        {
            self.authenticate().await?;
        }
        Ok(())
    }

    /// Fetch a raw authenticated response for the opt-in API inspection tool.
    /// Normal application code should use the typed methods instead.
    pub(crate) async fn get(
        &mut self,
        path: &str,
        params: Option<&[(&str, String)]>,
    ) -> Result<Value> {
        self.ensure_authenticated().await?;
        match self.get_authenticated(path, params).await {
            Err(error)
                if error
                    .chain()
                    .any(|cause| cause.downcast_ref::<AuthenticationExpired>().is_some()) =>
            {
                self.access_token = None;
                self.ensure_authenticated()
                    .await
                    .with_context(|| format!("re-authenticating before GET {path}"))?;
                self.get_authenticated(path, params)
                    .await
                    .with_context(|| format!("retrying GET {path}"))
            }
            result => result,
        }
    }

    pub(crate) async fn get_authenticated(
        &self,
        path: &str,
        params: Option<&[(&str, String)]>,
    ) -> Result<Value> {
        let query = params.map(|items| {
            items
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>()
        });
        let response = self
            .request("GET", path, query.as_deref(), None)
            .await
            .with_context(|| format!("GET {path}"))?;
        if response.get("success").and_then(Value::as_bool) != Some(true) {
            if response_indicates_expired(&response) {
                return Err(anyhow!(AuthenticationExpired));
            }
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
        for attempt in 0..3 {
            let response = request
                .try_clone()
                .ok_or_else(|| anyhow!("could not clone SunSynk request"))?
                .send()
                .await
                .with_context(|| format!("sending {method} {path}"))?;
            if is_transient_status(response.status()) && attempt < 2 {
                tokio::time::sleep(Duration::from_millis(250 * 2_u64.pow(attempt))).await;
                continue;
            }
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
            return Ok(body);
        }
        unreachable!("bounded HTTP retry loop must return")
    }
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn response_indicates_expired(response: &Value) -> bool {
    let message = response
        .get("msg")
        .or_else(|| response.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    [
        "token expired",
        "token invalid",
        "access token",
        "authentication expired",
        "unauthorized",
        "未登录",
        "过期",
    ]
    .iter()
    .any(|marker| message.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::is_transient_status;

    #[test]
    fn retries_rate_limits_and_server_failures_only() {
        assert!(is_transient_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_transient_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(!is_transient_status(reqwest::StatusCode::BAD_REQUEST));
        assert!(!is_transient_status(reqwest::StatusCode::UNAUTHORIZED));
    }
}
