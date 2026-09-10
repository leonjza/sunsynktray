use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{mpsc, Mutex, OnceLock},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Preferences {
    #[serde(default)]
    pub(crate) always_on_top: bool,
    #[serde(default = "default_refresh_seconds")]
    pub(crate) refresh_seconds: u64,
    #[serde(default = "default_history_days")]
    pub(crate) history_days: u64,
    #[serde(default)]
    pub(crate) tray_metric: Option<String>,
    #[serde(default)]
    pub(crate) selected_serial: Option<String>,
    #[serde(default)]
    pub(crate) compact_view: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            always_on_top: false,
            refresh_seconds: default_refresh_seconds(),
            history_days: default_history_days(),
            tray_metric: None,
            selected_serial: None,
            compact_view: false,
        }
    }
}

fn default_refresh_seconds() -> u64 {
    60
}

fn default_history_days() -> u64 {
    365
}

fn path() -> Result<PathBuf> {
    Ok(crate::platform::app_data_dir()?.join("settings.json"))
}

pub(crate) fn load() -> Result<Option<Preferences>> {
    let path = path()?;
    match std::fs::read_to_string(&path) {
        Ok(contents) => Ok(Some(serde_json::from_str(&contents)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn save_unlocked(preferences: &Preferences) -> Result<()> {
    let path = path()?;
    let directory = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("settings path has no parent directory"))?;
    std::fs::create_dir_all(directory)?;
    let temporary = directory.join(format!("settings.json.{}.tmp", std::process::id()));
    std::fs::write(&temporary, serde_json::to_vec_pretty(preferences)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn settings_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

type SettingsUpdate = Box<dyn FnOnce(&mut Preferences) + Send + 'static>;

enum SettingsCommand {
    Update(SettingsUpdate),
    Flush(mpsc::Sender<()>),
}

fn settings_sender() -> Option<&'static mpsc::Sender<SettingsCommand>> {
    static SENDER: OnceLock<Option<mpsc::Sender<SettingsCommand>>> = OnceLock::new();
    SENDER
        .get_or_init(|| {
            let (sender, receiver) = mpsc::channel();
            let result = std::thread::Builder::new()
                .name("suntray-settings".into())
                .spawn(move || {
                    while let Ok(command) = receiver.recv() {
                        match command {
                            SettingsCommand::Update(update) => {
                                let result = (|| {
                                    let _guard = settings_lock()
                                        .lock()
                                        .unwrap_or_else(|error| error.into_inner());
                                    let mut preferences = load()?.unwrap_or_default();
                                    update(&mut preferences);
                                    save_unlocked(&preferences)
                                })();
                                if let Err(error) = result {
                                    tracing::warn!(%error, "could not save settings");
                                }
                            }
                            SettingsCommand::Flush(done) => {
                                let _ = done.send(());
                            }
                        }
                    }
                });
            match result {
                Ok(_) => Some(sender),
                Err(error) => {
                    tracing::warn!(%error, "could not start settings writer");
                    None
                }
            }
        })
        .as_ref()
}

pub(crate) fn save_always_on_top_async(always_on_top: bool) {
    update_async(move |preferences| preferences.always_on_top = always_on_top);
}

pub(crate) fn save_refresh_seconds_async(refresh_seconds: u64) {
    update_async(move |preferences| preferences.refresh_seconds = refresh_seconds);
}

pub(crate) fn save_history_days_async(history_days: u64) {
    update_async(move |preferences| preferences.history_days = history_days);
}

pub(crate) fn save_tray_metric_async(tray_metric: Option<String>) {
    update_async(move |preferences| preferences.tray_metric = tray_metric);
}

pub(crate) fn save_selected_serial_async(selected_serial: String) {
    update_async(move |preferences| preferences.selected_serial = Some(selected_serial));
}

pub(crate) fn save_compact_view_async(compact_view: bool) {
    update_async(move |preferences| preferences.compact_view = compact_view);
}

fn update_async(update: impl FnOnce(&mut Preferences) + Send + 'static) {
    if let Some(sender) = settings_sender() {
        if let Err(error) = sender.send(SettingsCommand::Update(Box::new(update))) {
            tracing::warn!(%error, "could not queue settings update");
        }
    }
}

pub(crate) fn flush() {
    let Some(sender) = settings_sender() else {
        return;
    };
    let (done, completed) = mpsc::channel();
    if let Err(error) = sender.send(SettingsCommand::Flush(done)) {
        tracing::warn!(%error, "could not flush settings");
        return;
    }
    if completed.recv().is_err() {
        tracing::warn!("settings writer stopped before flushing");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip() {
        let preferences = Preferences {
            always_on_top: true,
            refresh_seconds: 30,
            history_days: 90,
            tray_metric: Some("power".into()),
            selected_serial: Some("abc".into()),
            compact_view: true,
        };
        let json = serde_json::to_string(&preferences).unwrap();
        assert_eq!(
            serde_json::from_str::<Preferences>(&json).unwrap(),
            preferences
        );
    }

    #[test]
    fn missing_preferences_use_defaults() {
        let preferences: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(preferences.refresh_seconds, 60);
        assert_eq!(preferences.history_days, 365);
        assert!(!preferences.always_on_top);
        assert!(!preferences.compact_view);
    }
}
