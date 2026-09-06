use crate::{
    app::polling::protocol::Command as PollCommand, domain::InverterSummary, storage::credentials,
};
use gpui_kit::*;
use std::sync::{atomic::AtomicU64, Arc};

use super::{ConnectionState, MonitorState, TrayMetric};

pub(crate) struct MonitorController {
    pub(crate) state: Arc<MonitorState>,
    pub(crate) fetching: bool,
    pub(crate) polling: bool,
    pub(crate) poll_generation: u64,
    pub(crate) connect_generation: u64,
    pub(crate) connect_epoch: Arc<AtomicU64>,
    pub(crate) connect_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    pub(crate) poll_sender: Option<tokio::sync::mpsc::Sender<PollCommand>>,
    pub(crate) poll_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    pub(crate) connection: ConnectionState,
    pub(crate) inverters: Vec<InverterSummary>,
    pub(crate) selected_serial: Option<String>,
    pub(crate) credentials: Option<(String, String)>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) refresh_seconds: u64,
    pub(crate) activity: String,
    pub(crate) next_refresh_in: Option<u64>,
    pub(crate) refresh_generation: u64,
    pub(crate) has_cached_data: bool,
    pub(crate) tray_metric: Option<TrayMetric>,
    pub(crate) history_date: chrono::NaiveDate,
    pub(crate) history_is_manual: bool,
    pub(crate) history_previous_date: Option<chrono::NaiveDate>,
    pub(crate) credentials_loaded: bool,
}

pub(crate) struct MonitorControllerGlobal(pub Entity<MonitorController>);
impl Global for MonitorControllerGlobal {}

impl MonitorController {
    pub(crate) fn new(state: Arc<MonitorState>) -> Self {
        let has_cached_data = state.has_live_data();
        Self {
            state,
            fetching: false,
            polling: false,
            poll_generation: 0,
            connect_generation: 0,
            connect_epoch: Arc::new(AtomicU64::new(0)),
            connect_cancel: None,
            poll_sender: None,
            poll_cancel: None,
            connection: ConnectionState::Unconfigured,
            inverters: Vec::new(),
            selected_serial: None,
            credentials: None,
            refresh_token: None,
            refresh_seconds: 60,
            activity: if has_cached_data {
                "Starting…"
            } else {
                "Loading saved account…"
            }
            .into(),
            next_refresh_in: None,
            refresh_generation: 0,
            has_cached_data,
            tray_metric: None,
            history_date: chrono::Local::now().date_naive(),
            history_is_manual: false,
            history_previous_date: None,
            credentials_loaded: false,
        }
    }

    pub(crate) fn initialize(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(async { credentials::load() })
                .await;
            entity.update(cx, |controller, cx| {
                controller.credentials_loaded = true;
                match result {
                    Ok(Some(saved)) => {
                        let email = saved.email.clone();
                        let password = saved.password.clone();
                        controller.connection = if controller.has_cached_data {
                            ConnectionState::Connected
                        } else {
                            ConnectionState::Connecting
                        };
                        controller.selected_serial = saved.selected_serial;
                        controller.credentials = Some((email.clone(), password.clone()));
                        controller.refresh_token = saved.refresh_token;
                        controller.refresh_seconds =
                            saved.refresh_seconds.unwrap_or(60).clamp(1, 3600);
                        controller.tray_metric =
                            TrayMetric::from_saved(saved.tray_metric.as_deref());
                        controller.activity = if controller.has_cached_data {
                            "Reconnecting…"
                        } else {
                            "Starting…"
                        }
                        .into();
                        controller.connect(email, password, cx);
                    }
                    Ok(None) => {
                        controller.connection = ConnectionState::Unconfigured;
                        controller.activity = "No account configured".into();
                        controller.update_tray(cx);
                    }
                    Err(error) => {
                        tracing::warn!(%error, "could not read saved SunSynk credentials");
                        controller.connection = ConnectionState::Unconfigured;
                        controller.activity = "No account configured".into();
                        controller.update_tray(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn set_tray_metric(&mut self, metric: Option<TrayMetric>, cx: &mut Context<Self>) {
        self.tray_metric = metric;
        credentials::save_tray_metric_async(metric.map(TrayMetric::saved_name).map(str::to_owned));
        self.update_tray(cx);
        cx.notify();
    }

    pub(crate) fn update_tray(&self, cx: &mut App) {
        let snapshot = self.state.snapshot();
        let connected = matches!(
            self.connection,
            ConnectionState::Connected | ConnectionState::Stale
        );
        let value = connected
            .then(|| self.tray_metric.map(|metric| metric.value(&snapshot)))
            .flatten();
        let symbol = match (connected, self.tray_metric) {
            (false, _) => "bolt.slash.fill",
            (true, Some(TrayMetric::Soc)) => "battery.100",
            (true, Some(TrayMetric::Load)) => "house.fill",
            (true, Some(TrayMetric::Solar)) => "sun.max.fill",
            (true, None) => "bolt.fill",
        };
        let inverter_name = self
            .selected_serial
            .as_ref()
            .and_then(|serial| {
                self.inverters
                    .iter()
                    .find(|inverter| &inverter.serial == serial)
            })
            .map(|inverter| {
                if inverter.alias.is_empty() {
                    inverter.serial.as_str()
                } else {
                    inverter.alias.as_str()
                }
            })
            .unwrap_or("SunSynk");
        let tooltip = match value.as_deref() {
            Some(value) => format!("{inverter_name} · {value}"),
            None if connected => inverter_name.to_owned(),
            None => format!("{inverter_name} · Disconnected"),
        };
        crate::platform::tray::update(cx, value.as_deref(), symbol, &tooltip);
    }

    pub(crate) fn send_poll_command(
        &mut self,
        command: PollCommand,
        cx: &mut Context<Self>,
    ) -> bool {
        let fetch = matches!(command, PollCommand::Refresh | PollCommand::Select(_, _));
        if !crate::app::polling::should_queue_command(&command, self.fetching) {
            return false;
        }
        if let Some(sender) = &self.poll_sender {
            if sender.try_send(command).is_ok() {
                if fetch {
                    self.fetching = true;
                    self.refresh_generation = self.refresh_generation.wrapping_add(1);
                    self.next_refresh_in = None;
                    self.activity = "Fetching new data…".into();
                }
                cx.notify();
                return true;
            }
        }
        false
    }
}

impl Drop for MonitorController {
    fn drop(&mut self) {
        if let Some(sender) = self.poll_sender.take() {
            let _ = sender.try_send(PollCommand::Stop);
        }
        if let Some(cancel) = self.poll_cancel.take() {
            let _ = cancel.send(());
        }
    }
}
