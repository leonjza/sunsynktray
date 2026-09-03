use super::error::{AuthenticationExpired, RefreshTokenRejected};
use super::{parsing::unsigned, SunsynkClient};
use anyhow::{anyhow, bail, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use md5::{Digest, Md5};
use rsa::{pkcs1v15::Pkcs1v15Encrypt, pkcs8::DecodePublicKey, RsaPublicKey};
use serde_json::Value;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

impl SunsynkClient {
    pub(crate) async fn connect(&mut self) -> Result<()> {
        self.report_progress("Fetching SunSynk public key…");
        let public_key = self.public_key().await?;
        // The web app currently returns base64-encoded DER (SubjectPublicKeyInfo),
        // although some deployments have returned PEM in the past.
        let key = if public_key.contains("BEGIN PUBLIC KEY") {
            RsaPublicKey::from_public_key_pem(&public_key)
        } else {
            let der = BASE64
                .decode(public_key.trim())
                .context("SunSynk returned malformed public-key base64")?;
            RsaPublicKey::from_public_key_der(&der)
        }
        .context("SunSynk returned an invalid public key")?;
        self.report_progress("Encrypting password…");
        let encrypted = key.encrypt(
            &mut rsa::rand_core::OsRng,
            Pkcs1v15Encrypt,
            self.password.as_bytes(),
        )?;
        let nonce = nonce();
        let sign = md5_hex(&format!(
            "nonce={nonce}&source=sunsynk{}",
            &public_key[..10.min(public_key.len())]
        ));
        self.report_progress("Logging in to SunSynk…");
        let body = self.request("POST", "/oauth/token/new", None, Some(&serde_json::json!({ "username": self.username, "password": BASE64.encode(encrypted), "grant_type": "password", "client_id": "csp-web", "source": "sunsynk", "nonce": nonce, "sign": sign }))).await?;
        let data = body.get("data").cloned().unwrap_or(Value::Null);
        self.access_token = data
            .get("access_token")
            .and_then(Value::as_str)
            .map(str::to_owned);
        self.access_expires_at = data
            .get("expires_in")
            .and_then(unsigned)
            .map(|seconds| Instant::now() + Duration::from_secs(seconds));
        self.refresh_token = data
            .get("refresh_token")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| self.refresh_token.clone());
        if body.get("success").and_then(Value::as_bool) != Some(true) || self.access_token.is_none()
        {
            bail!(
                "{}",
                body.get("msg")
                    .and_then(Value::as_str)
                    .unwrap_or("SunSynk rejected the login")
            );
        }
        Ok(())
    }

    async fn public_key(&self) -> Result<String> {
        let n = nonce();
        let value = self
            .request(
                "GET",
                "/anonymous/publicKey",
                Some(&[
                    ("nonce", n.to_string()),
                    ("source", "sunsynk".into()),
                    (
                        "sign",
                        md5_hex(&format!("nonce={n}&source=sunsynkPOWER_VIEW")),
                    ),
                ]),
                None,
            )
            .await?;
        value
            .get("data")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| anyhow!("SunSynk did not return a public key"))
    }
    pub(crate) async fn authenticate(&mut self) -> Result<()> {
        if let Some(token) = self.refresh_token.clone() {
            self.report_progress("Refreshing SunSynk access token…");
            match self.refresh(&token).await {
                Ok(()) => return Ok(()),
                Err(error) if error.downcast_ref::<RefreshTokenRejected>().is_some() => {
                    self.access_token = None;
                }
                Err(error) => {
                    return Err(error).context("refreshing the SunSynk access token");
                }
            }
        }
        self.connect().await.context("authenticating with SunSynk")
    }

    async fn refresh(&mut self, token: &str) -> Result<()> {
        let body = match self
            .request(
                "POST",
                "/oauth/token/new",
                None,
                Some(&serde_json::json!({
                    "grant_type": "refresh_token",
                    "refresh_token": token,
                    "client_id": "csp-web",
                    "source": "sunsynk",
                })),
            )
            .await
        {
            Err(error) if error.downcast_ref::<AuthenticationExpired>().is_some() => {
                return Err(anyhow!(RefreshTokenRejected));
            }
            Err(error) => return Err(error.context("requesting a refreshed SunSynk token")),
            Ok(body) => body,
        };
        if body.get("success").and_then(Value::as_bool) != Some(true) {
            return Err(anyhow!(RefreshTokenRejected));
        }
        let data = body.get("data").cloned().unwrap_or(Value::Null);
        self.access_token = data
            .get("access_token")
            .and_then(Value::as_str)
            .map(str::to_owned);
        self.access_expires_at = data
            .get("expires_in")
            .and_then(unsigned)
            .map(|seconds| Instant::now() + Duration::from_secs(seconds));
        self.refresh_token = data
            .get("refresh_token")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| Some(token.to_owned()));
        if self.access_token.is_none() {
            return Err(anyhow!(RefreshTokenRejected));
        }
        Ok(())
    }
}

fn nonce() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn md5_hex(value: &str) -> String {
    format!("{:x}", Md5::digest(value.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::super::error::{AuthenticationExpired, RefreshTokenRejected};
    use anyhow::anyhow;

    #[test]
    fn authentication_errors_remain_machine_classifiable() {
        let error = anyhow!(AuthenticationExpired);
        assert!(error.downcast_ref::<AuthenticationExpired>().is_some());
        assert!(error.downcast_ref::<RefreshTokenRejected>().is_none());
    }
}
