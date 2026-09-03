use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    storage::{config::Settings, credentials},
    sunsynk::SunsynkClient,
};

#[derive(Serialize)]
struct ApiFixture {
    captured_at: String,
    api_base_url: String,
    account: String,
    inverter_serial: String,
    plant_id: Option<i64>,
    responses: BTreeMap<String, Value>,
}

pub(crate) fn run(args: &[String], settings: Settings) -> Result<()> {
    let saved = credentials::load()?.context("no SunSynk credentials found in the keychain")?;
    let serial_override = option(args, "--serial");
    let output = option(args, "--output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("fixtures/api/latest.json"));

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("could not create diagnostic runtime")?;
    let fixture = runtime.block_on(async {
        let mut client = SunsynkClient::new(
            settings.api_base_url.clone(),
            saved.email.clone(),
            saved.password,
        )?
        .with_refresh_token(saved.refresh_token);

        let inverters = client.list_inverters().await?;
        if inverters.is_empty() {
            bail!("SunSynk returned no inverters");
        }
        let selected = serial_override
            .as_deref()
            .or(saved.selected_serial.as_deref())
            .and_then(|serial| inverters.iter().find(|item| item.serial == serial))
            .or_else(|| inverters.first());
        let selected = selected.context("could not select an inverter")?;
        let serial = selected.serial.clone();
        let plant_id = selected.plant_id;

        println!("Available inverters:");
        for inverter in &inverters {
            println!("  {}  {}", inverter.serial, inverter.alias);
        }
        println!("Inspecting: {}", serial);

        let mut responses = BTreeMap::new();
        if let Some(plant_id) = plant_id {
            let date = Utc::now().date_naive().to_string();
            responses.insert(
                "plant_realtime".into(),
                client
                    .inspect_endpoint(
                        &format!("/api/v1/plant/{plant_id}/realtime"),
                        Some(&[("id", plant_id.to_string())]),
                    )
                    .await?,
            );
            responses.insert(
                "plant_energy_day".into(),
                client
                    .inspect_endpoint(
                        &format!("/api/v1/plant/energy/{plant_id}/day"),
                        Some(&[
                            ("lan", "en".into()),
                            ("date", date.clone()),
                            ("id", plant_id.to_string()),
                        ]),
                    )
                    .await?,
            );
            responses.insert(
                "plant_energy_flow".into(),
                client
                    .inspect_endpoint(
                        &format!("/api/v1/plant/energy/{plant_id}/flow"),
                        Some(&[("date", date)]),
                    )
                    .await?,
            );
        }

        Ok::<_, anyhow::Error>(ApiFixture {
            captured_at: Utc::now().to_rfc3339(),
            api_base_url: settings.api_base_url.clone(),
            account: "[REDACTED]".into(),
            inverter_serial: serial,
            plant_id,
            responses: responses
                .into_iter()
                .map(|(name, response)| (name, redact(response)))
                .collect(),
        })
    })?;

    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).context("could not create fixture directory")?;
    }
    let json = serde_json::to_vec_pretty(&fixture)?;
    fs::write(&output, json).context("could not write API fixture")?;
    println!("Wrote redacted API fixture to {}", output.display());
    Ok(())
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .find_map(|arg| arg.strip_prefix(&format!("{name}=")))
        .map(str::to_owned)
}

fn redact(value: Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(key, value)| {
                    let redacted = if is_sensitive_key(&key) {
                        Value::String("[REDACTED]".into())
                    } else {
                        redact(value)
                    };
                    (key, redacted)
                })
                .collect::<Map<_, _>>(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(redact).collect()),
        value => value,
    }
}

fn is_sensitive_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "password" | "access_token" | "refresh_token" | "token" | "authorization" | "email"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redaction_removes_credentials_recursively() {
        let value = redact(json!({
            "data": {"access_token": "secret", "value": 1},
            "records": [{"refresh_token": "also-secret", "value": 2}]
        }));
        assert_eq!(value["data"]["access_token"], "[REDACTED]");
        assert_eq!(value["records"][0]["refresh_token"], "[REDACTED]");
        assert_eq!(value["data"]["value"], 1);
    }
}
