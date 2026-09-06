use anyhow::Result;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt,
    sync::{mpsc, Arc, Condvar, Mutex, OnceLock},
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

fn update_record(
    email: Option<&str>,
    update: impl FnOnce(&mut SavedCredentials) -> bool,
) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut record = load_unlocked()?.ok_or_else(|| anyhow::anyhow!("no saved credentials"))?;
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

pub(crate) fn save_refresh_seconds(email: &str, refresh_seconds: u64) -> Result<()> {
    update_record(Some(email), |record| {
        let refresh_seconds = refresh_seconds.clamp(1, 3600);
        if record.refresh_seconds == Some(refresh_seconds) {
            false
        } else {
            record.refresh_seconds = Some(refresh_seconds);
            true
        }
    })
}

type PersistenceTaskFn = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersistenceKey {
    Credentials,
    Selection,
    TrayMetric,
    RefreshToken,
    RefreshSeconds,
    Flush,
}

struct PersistenceTask {
    key: PersistenceKey,
    task: PersistenceTaskFn,
}

struct PersistenceQueue {
    pending: Mutex<VecDeque<PersistenceTask>>,
    wake: Condvar,
}

impl PersistenceQueue {
    fn enqueue(&self, key: PersistenceKey, task: PersistenceTaskFn) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if key != PersistenceKey::Flush {
            pending.retain(|item| item.key != key);
        }
        pending.push_back(PersistenceTask { key, task });
        self.wake.notify_one();
    }

    fn next(&self) -> PersistenceTask {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        loop {
            if let Some(task) = pending.pop_front() {
                return task;
            }
            pending = self
                .wake
                .wait(pending)
                .unwrap_or_else(|error| error.into_inner());
        }
    }
}

fn persistence_queue() -> Option<&'static Arc<PersistenceQueue>> {
    static QUEUE: OnceLock<Option<Arc<PersistenceQueue>>> = OnceLock::new();
    QUEUE
        .get_or_init(|| {
            let queue = Arc::new(PersistenceQueue {
                pending: Mutex::new(VecDeque::new()),
                wake: Condvar::new(),
            });
            let worker_queue = queue.clone();
            let worker = std::thread::Builder::new()
                .name("suntray-credentials".into())
                .spawn(move || loop {
                    (worker_queue.next().task)();
                });
            match worker {
                Ok(_) => Some(queue),
                Err(error) => {
                    tracing::error!(%error, "could not start credential persistence worker");
                    None
                }
            }
        })
        .as_ref()
}

fn enqueue_persistence(key: PersistenceKey, task: impl FnOnce() + Send + 'static) -> bool {
    if let Some(queue) = persistence_queue() {
        queue.enqueue(key, Box::new(task));
        true
    } else {
        tracing::warn!("credential persistence worker could not be started");
        false
    }
}

pub(crate) fn flush() {
    let (sender, receiver) = mpsc::sync_channel(0);
    if !enqueue_persistence(PersistenceKey::Flush, move || {
        let _ = sender.send(());
    }) {
        return;
    }
    if receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .is_err()
    {
        tracing::warn!("credential persistence worker stopped before flushing");
    }
}

pub(crate) fn save_selection_async(email: String, serial: String) {
    enqueue_persistence(PersistenceKey::Selection, move || {
        if let Err(error) = save_selection(&email, &serial) {
            tracing::warn!(%error, "could not save selected inverter");
        }
    });
}

pub(crate) fn save_tray_metric_async(metric: Option<String>) {
    enqueue_persistence(PersistenceKey::TrayMetric, move || {
        if let Err(error) = save_tray_metric(metric.as_deref()) {
            tracing::warn!(%error, "could not save tray metric");
        }
    });
}

pub(crate) fn save_refresh_token_async(email: String, token: String) {
    enqueue_persistence(PersistenceKey::RefreshToken, move || {
        if let Err(error) = save_refresh_token(&email, &token) {
            tracing::warn!(%error, "could not persist refreshed SunSynk token");
        }
    });
}

pub(crate) fn save_refresh_seconds_async(email: String, refresh_seconds: u64) {
    enqueue_persistence(PersistenceKey::RefreshSeconds, move || {
        if let Err(error) = save_refresh_seconds(&email, refresh_seconds) {
            tracing::warn!(%error, "could not persist refresh interval");
        }
    });
}

pub(crate) fn save_async(
    email: String,
    password: String,
    refresh_token: Option<String>,
    selected_serial: Option<String>,
    refresh_seconds: u64,
    tray_metric: Option<String>,
) {
    enqueue_persistence(PersistenceKey::Credentials, move || {
        if let Err(error) = save(
            &email,
            &password,
            refresh_token.as_deref(),
            selected_serial.as_deref(),
            refresh_seconds,
            tray_metric.as_deref(),
        ) {
            tracing::warn!(%error, "could not save SunSynk credentials");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{PersistenceKey, PersistenceQueue};
    use std::{
        collections::VecDeque,
        sync::{Condvar, Mutex},
    };

    #[test]
    fn flush_barriers_are_not_coalesced() {
        let queue = PersistenceQueue {
            pending: Mutex::new(VecDeque::new()),
            wake: Condvar::new(),
        };
        queue.enqueue(PersistenceKey::Selection, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Selection, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Flush, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Flush, Box::new(|| {}));

        let pending = queue.pending.lock().unwrap();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].key, PersistenceKey::Selection);
        assert_eq!(pending[1].key, PersistenceKey::Flush);
        assert_eq!(pending[2].key, PersistenceKey::Flush);
    }
}
