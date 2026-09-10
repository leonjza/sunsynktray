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

pub(crate) fn save(email: &str, password: &str, refresh_token: Option<&str>) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    save_record_unlocked(&SavedCredentials {
        email: email.into(),
        password: password.into(),
        refresh_token: refresh_token.map(str::to_owned),
    })
}

pub(crate) fn save_refresh_token(
    email: &str,
    expected_token: Option<&str>,
    token: &str,
) -> Result<()> {
    let _guard = keychain_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let mut record = load_unlocked()?.ok_or_else(|| anyhow::anyhow!("no saved credentials"))?;
    if record.email != email {
        tracing::debug!(%email, "ignoring stale refresh-token update for another account");
        return Ok(());
    }
    if record.refresh_token.as_deref() != expected_token {
        tracing::debug!("ignoring stale refresh-token update");
        return Ok(());
    }
    if record.refresh_token.as_deref() == Some(token) {
        return Ok(());
    }
    record.refresh_token = Some(token.to_owned());
    save_record_unlocked(&record)
}

type PersistenceTaskFn = Box<dyn FnOnce() + Send + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersistenceKey {
    Credentials,
    RefreshToken,
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
        if key != PersistenceKey::Flush && key != PersistenceKey::RefreshToken {
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

pub(crate) fn save_refresh_token_async(
    email: String,
    expected_token: Option<String>,
    token: String,
) {
    enqueue_persistence(PersistenceKey::RefreshToken, move || {
        if let Err(error) = save_refresh_token(&email, expected_token.as_deref(), &token) {
            tracing::warn!(%error, "could not persist refreshed SunSynk token");
        }
    });
}

pub(crate) fn save_async(email: String, password: String, refresh_token: Option<String>) {
    enqueue_persistence(PersistenceKey::Credentials, move || {
        if let Err(error) = save(&email, &password, refresh_token.as_deref()) {
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
        queue.enqueue(PersistenceKey::Credentials, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Credentials, Box::new(|| {}));
        queue.enqueue(PersistenceKey::RefreshToken, Box::new(|| {}));
        queue.enqueue(PersistenceKey::RefreshToken, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Flush, Box::new(|| {}));
        queue.enqueue(PersistenceKey::Flush, Box::new(|| {}));

        let pending = queue.pending.lock().unwrap();
        assert_eq!(pending.len(), 5);
        assert_eq!(pending[0].key, PersistenceKey::Credentials);
        assert_eq!(pending[1].key, PersistenceKey::RefreshToken);
        assert_eq!(pending[2].key, PersistenceKey::RefreshToken);
        assert_eq!(pending[3].key, PersistenceKey::Flush);
        assert_eq!(pending[4].key, PersistenceKey::Flush);
    }
}
