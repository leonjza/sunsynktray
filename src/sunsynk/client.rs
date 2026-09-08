use super::error::AuthenticationExpired;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

type ProgressCallback = Arc<dyn Fn(&str) + Send + Sync>;
type RequestLogCallback = Arc<dyn Fn(String) + Send + Sync>;
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
pub(crate) struct AuthState {
    pub(crate) access_token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) access_expires_at: Option<Instant>,
}

#[derive(Clone)]
pub(crate) struct SunsynkClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) auth: Arc<Mutex<AuthState>>,
    pub(crate) auth_refresh: Arc<tokio::sync::Mutex<()>>,
    pub(crate) progress: Option<ProgressCallback>,
    pub(crate) request_log: Option<RequestLogCallback>,
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
            auth: Arc::new(Mutex::new(AuthState::default())),
            auth_refresh: Arc::new(tokio::sync::Mutex::new(())),
            progress: None,
            request_log: None,
        })
    }

    pub(crate) fn with_refresh_token(self, token: Option<String>) -> Self {
        self.auth
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .refresh_token = token;
        self
    }

    pub(crate) fn auth_state(&self) -> Arc<Mutex<AuthState>> {
        Arc::clone(&self.auth)
    }

    pub(crate) fn with_auth_state(mut self, auth: Arc<Mutex<AuthState>>) -> Self {
        self.auth = auth;
        self
    }

    pub(crate) fn refresh_token(&self) -> Option<String> {
        self.auth
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .refresh_token
            .clone()
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

    pub(crate) fn with_request_log<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.request_log = Some(Arc::new(callback));
        self
    }

    pub(crate) async fn ensure_authenticated(&self) -> Result<()> {
        let _refresh_guard = self.auth_refresh.lock().await;
        let needs_auth = {
            let auth = self.auth.lock().unwrap_or_else(|error| error.into_inner());
            auth.access_token.is_none()
                || auth.access_expires_at.is_some_and(|expires| {
                    expires.saturating_duration_since(Instant::now()) <= Duration::from_secs(30)
                })
        };
        if needs_auth {
            self.authenticate().await?;
        }
        Ok(())
    }

    /// Fetch a raw authenticated response for the opt-in API inspection tool.
    /// Normal application code should use the typed methods instead.
    pub(crate) async fn get(&self, path: &str, params: Option<&[(&str, String)]>) -> Result<Value> {
        self.ensure_authenticated().await?;
        match self.get_authenticated(path, params).await {
            Err(error)
                if error
                    .chain()
                    .any(|cause| cause.downcast_ref::<AuthenticationExpired>().is_some()) =>
            {
                self.auth
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .access_token = None;
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
                self.log_request(format!("GET {path} rejected: authentication expired"));
                return Err(anyhow!(AuthenticationExpired));
            }
            let message = response
                .get("msg")
                .and_then(Value::as_str)
                .unwrap_or("SunSynk API request failed");
            self.log_request(format!("GET {path} rejected by API: {message}"));
            bail!("GET {path}: {}", message);
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
        let request_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let started = Instant::now();
        const MAX_REQUEST_DURATION: Duration = Duration::from_secs(60);
        let mut request = self
            .http
            .request(method.parse()?, format!("{}{}", self.base_url, path))
            .header("Accept", "application/json");
        if let Some(token) = self
            .auth
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .access_token
            .clone()
        {
            request = request.bearer_auth(token);
        }
        if let Some(params) = params {
            request = request.query(params);
        }
        if let Some(json) = json {
            request = request.json(json);
        }
        for attempt in 0..3 {
            let Some(remaining) = MAX_REQUEST_DURATION.checked_sub(started.elapsed()) else {
                let error = anyhow!("SunSynk request deadline exceeded for {method} {path}");
                self.log_request(format!("#{request_id} {method} {path} timed out after 60s"));
                return Err(error);
            };
            let response = match tokio::time::timeout(
                remaining,
                request
                    .try_clone()
                    .ok_or_else(|| anyhow!("could not clone SunSynk request"))?
                    .send(),
            )
            .await
            {
                Ok(Ok(response)) => response,
                Ok(Err(error)) => {
                    self.log_request(format!(
                        "#{request_id} {method} {path} failed after {}ms: {error}",
                        started.elapsed().as_millis()
                    ));
                    return Err(error).with_context(|| format!("sending {method} {path}"));
                }
                Err(_) => {
                    let error = anyhow!("SunSynk request deadline exceeded for {method} {path}");
                    self.log_request(format!(
                        "#{request_id} {method} {path} timed out after {}ms",
                        started.elapsed().as_millis()
                    ));
                    return Err(error);
                }
            };
            if is_transient_status(response.status()) && attempt < 2 {
                self.log_request(format!(
                    "#{request_id} {method} {path} attempt {} got {} · retrying",
                    attempt + 1,
                    response.status()
                ));
                let delay = retry_delay(response.headers(), attempt).min(remaining);
                tokio::time::sleep(delay).await;
                continue;
            }
            if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                self.log_request(format!(
                    "#{request_id} {method} {path} failed with 401 after {}ms",
                    started.elapsed().as_millis()
                ));
                return Err(anyhow!(AuthenticationExpired));
            }
            let status = response.status();
            if !status.is_success() {
                self.log_request(format!(
                    "#{request_id} {method} {path} failed with {status} after {}ms",
                    started.elapsed().as_millis()
                ));
            }
            let response = response
                .error_for_status()
                .with_context(|| format!("SunSynk returned an HTTP error for {method} {path}"))?;
            let body: Value = match response.json().await {
                Ok(body) => body,
                Err(error) => {
                    self.log_request(format!(
                        "#{request_id} {method} {path} returned invalid JSON after {}ms: {error}",
                        started.elapsed().as_millis()
                    ));
                    return Err(error)
                        .with_context(|| format!("decoding SunSynk response for {method} {path}"));
                }
            };
            self.log_request(format!(
                "#{request_id} {method} {path} succeeded with {} after {}ms",
                status,
                started.elapsed().as_millis()
            ));
            return Ok(body);
        }
        unreachable!("bounded HTTP retry loop must return")
    }

    fn log_request(&self, event: String) {
        if let Some(callback) = &self.request_log {
            callback(event);
        }
    }
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn retry_delay(headers: &reqwest::header::HeaderMap, attempt: u32) -> Duration {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_retry_after)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_millis(250 * 2_u64.pow(attempt)))
        .min(Duration::from_secs(30))
}

fn parse_retry_after(value: &str) -> Option<u64> {
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(seconds);
    }

    let date = chrono::DateTime::parse_from_rfc2822(value).ok()?;
    let seconds = (date.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_seconds();
    Some(seconds.max(0) as u64)
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
    use super::{is_transient_status, parse_retry_after, retry_delay, Duration};
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    #[test]
    fn retries_rate_limits_and_server_failures_only() {
        assert!(is_transient_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_transient_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(!is_transient_status(reqwest::StatusCode::BAD_REQUEST));
        assert!(!is_transient_status(reqwest::StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn retry_after_is_honoured_and_bounded() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("10"));
        assert_eq!(retry_delay(&headers, 0), Duration::from_secs(10));
        headers.insert(RETRY_AFTER, HeaderValue::from_static("60"));
        assert_eq!(retry_delay(&headers, 0), Duration::from_secs(30));
        headers.insert(RETRY_AFTER, HeaderValue::from_static("invalid"));
        assert_eq!(retry_delay(&headers, 1), Duration::from_millis(500));
    }

    #[test]
    fn retry_after_accepts_http_dates() {
        assert!(parse_retry_after("Wed, 21 Oct 2099 07:28:00 GMT").is_some());
        assert_eq!(parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT"), Some(0));
    }
}
