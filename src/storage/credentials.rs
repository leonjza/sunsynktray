use anyhow::Result;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::{Mutex, OnceLock},
};

const SERVICE: &str = "com.suntray.sunsynk";
const ACCOUNT: &str = "account";

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SavedCredentials {
    pub(crate) email: String,
    pub(crate) password: String,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default)]
    pub(crate) selected_serial: Option<String>,
    #[serde(default)]
    pub(crate) refresh_seconds: Option<u64>,
    #[serde(default)]
    pub(crate) tray_metric: Option<String>,
}

impl fmt::Debug for SavedCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SavedCredentials")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("selected_serial", &self.selected_serial)
            .field("refresh_seconds", &self.refresh_seconds)
            .field("tray_metric", &self.tray_metric)
            .finish()
    }
}

fn entry() -> Result<Entry> {
    Ok(Entry::new(SERVICE, ACCOUNT)?)
}

fn keychain_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn load_unlocked() -> Result<Option<SavedCredentials>> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(serde_json::from_str(&secret)?)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(crate) fn load() -> Result<Option<SavedCredentials>> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    load_unlocked()
}

fn save_record_unlocked(record: &SavedCredentials) -> Result<()> {
    entry()?.set_password(&serde_json::to_string(record)?)?;
    Ok(())
}

fn empty_record(email: &str) -> SavedCredentials {
    SavedCredentials {
        email: email.to_owned(),
        password: String::new(),
        refresh_token: None,
        selected_serial: None,
        refresh_seconds: None,
        tray_metric: None,
    }
}

fn update_record(
    email: Option<&str>,
    update: impl FnOnce(&mut SavedCredentials) -> bool,
) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut record = load_unlocked()?.unwrap_or_else(|| empty_record(email.unwrap_or_default()));
    let previous_email = record.email.clone();
    if let Some(email) = email {
        record.email = email.to_owned();
    }
    if update(&mut record) || record.email != previous_email {
        save_record_unlocked(&record)
    } else {
        Ok(())
    }
}

pub(crate) fn save(
    email: &str,
    password: &str,
    refresh_token: Option<&str>,
    selected_serial: Option<&str>,
    refresh_seconds: u64,
    tray_metric: Option<&str>,
) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let existing = load_unlocked()?;
    let selected_serial = selected_serial.map(str::to_owned).or_else(|| {
        existing
            .as_ref()
            .and_then(|saved| (saved.email == email).then_some(saved.selected_serial.clone()))
            .flatten()
    });
    save_record_unlocked(&SavedCredentials {
        email: email.into(),
        password: password.into(),
        refresh_token: refresh_token.map(str::to_owned),
        selected_serial,
        refresh_seconds: Some(refresh_seconds),
        tray_metric: tray_metric.map(str::to_owned).or_else(|| {
            existing
                .as_ref()
                .and_then(|saved| saved.tray_metric.clone())
        }),
    })
}

pub(crate) fn save_selection(email: &str, serial: &str) -> Result<()> {
    update_record(Some(email), |record| {
        if record.selected_serial.as_deref() != Some(serial) {
            record.selected_serial = Some(serial.to_owned());
            true
        } else {
            false
        }
    })
}

pub(crate) fn save_tray_metric(metric: Option<&str>) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut record = load_unlocked()?.ok_or_else(|| anyhow::anyhow!("no saved credentials"))?;
    if record.tray_metric.as_deref() == metric {
        return Ok(());
    }
    record.tray_metric = metric.map(str::to_owned);
    save_record_unlocked(&record)
}

pub(crate) fn save_refresh_token(email: &str, token: &str) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut record = load_unlocked()?.ok_or_else(|| anyhow::anyhow!("no saved credentials"))?;
    if record.email == email && record.refresh_token.as_deref() == Some(token) {
        return Ok(());
    }
    record.email = email.to_owned();
    record.refresh_token = Some(token.to_owned());
    save_record_unlocked(&record)
}

pub(crate) fn save_selection_async(email: String, serial: String) {
    std::thread::spawn(move || {
        if let Err(error) = save_selection(&email, &serial) {
            tracing::warn!(%error, "could not save selected inverter");
        }
    });
}

pub(crate) fn save_tray_metric_async(metric: Option<String>) {
    std::thread::spawn(move || {
        if let Err(error) = save_tray_metric(metric.as_deref()) {
            tracing::warn!(%error, "could not save tray metric");
        }
    });
}

pub(crate) fn save_refresh_token_async(email: String, token: String) {
    std::thread::spawn(move || {
        if let Err(error) = save_refresh_token(&email, &token) {
            tracing::warn!(%error, "could not persist refreshed SunSynk token");
        }
    });
}
