use crate::{
    app::polling::protocol::Command as PollCommand, domain::InverterSummary, storage::credentials,
};
use gpui_kit::*;
use std::{
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use super::{ConnectionState, MonitorState, TrayMetric};

pub(crate) struct MonitorController {
    pub(crate) state: Arc<MonitorState>,
    pub(crate) fetching: bool,
    pub(crate) polling: bool,
    pub(crate) poll_generation: u64,
    pub(crate) connect_generation: u64,
    pub(crate) connect_epoch: Arc<AtomicU64>,
    pub(crate) poll_sender: Option<tokio::sync::mpsc::Sender<PollCommand>>,
    pub(crate) connection: ConnectionState,
    pub(crate) inverters: Vec<InverterSummary>,
    pub(crate) selected_serial: Option<String>,
    pub(crate) credentials: Option<(String, String)>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) refresh_seconds: u64,
    pub(crate) activity: String,
    pub(crate) next_refresh_in: Option<u64>,
    pub(crate) has_cached_data: bool,
    pub(crate) tray_metric: Option<TrayMetric>,
    pub(crate) history_date: chrono::NaiveDate,
    pub(crate) history_is_manual: bool,
    pub(crate) history_previous_date: Option<chrono::NaiveDate>,
}

pub(crate) struct MonitorControllerGlobal(pub Entity<MonitorController>);
impl Global for MonitorControllerGlobal {}

impl MonitorController {
    pub(crate) fn new(state: Arc<MonitorState>) -> Self {
        let saved = match credentials::load() {
            Ok(saved) => saved,
            Err(error) => {
                tracing::warn!(%error, "could not read saved SunSynk credentials");
                None
            }
        };
        let credentials = saved.clone().map(|saved| (saved.email, saved.password));
        let has_cached_data = state.has_live_data();
        let connection = if credentials.is_some() {
            if has_cached_data {
                ConnectionState::Connected
            } else {
                ConnectionState::Connecting
            }
        } else {
            ConnectionState::Unconfigured
        };
        Self {
            state,
            fetching: false,
            polling: false,
            poll_generation: 0,
            connect_generation: 0,
            connect_epoch: Arc::new(AtomicU64::new(0)),
            poll_sender: None,
            connection,
            inverters: Vec::new(),
            selected_serial: saved
                .as_ref()
                .and_then(|saved| saved.selected_serial.clone()),
            credentials,
            refresh_token: saved.as_ref().and_then(|saved| saved.refresh_token.clone()),
            refresh_seconds: saved
                .as_ref()
                .and_then(|saved| saved.refresh_seconds)
                .unwrap_or(60)
                .clamp(1, 3600),
            activity: if has_cached_data {
                "Reconnecting…"
            } else if saved.is_some() {
                "Starting…"
            } else {
                "No account configured"
            }
            .into(),
            next_refresh_in: None,
            has_cached_data,
            tray_metric: saved
                .as_ref()
                .and_then(|saved| TrayMetric::from_saved(saved.tray_metric.as_deref())),
            history_date: chrono::Local::now().date_naive(),
            history_is_manual: false,
            history_previous_date: None,
        }
    }

    pub(crate) fn initialize(&mut self, cx: &mut Context<Self>) {
        let Some((email, password)) = self.credentials.clone() else {
            self.update_tray(cx);
            return;
        };
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(10))
                .await;
            entity.update(cx, |controller, cx| controller.connect(email, password, cx));
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

    pub(crate) fn send_poll_command(&mut self, command: PollCommand, cx: &mut Context<Self>) {
        let fetch = matches!(command, PollCommand::Refresh | PollCommand::Select(_, _));
        if !crate::app::polling::should_queue_command(&command, self.fetching) {
            return;
        }
        if let Some(sender) = &self.poll_sender {
            if sender.try_send(command).is_ok() {
                if fetch {
                    self.fetching = true;
                    self.next_refresh_in = None;
                    self.activity = "Fetching new data…".into();
                }
                cx.notify();
            }
        }
    }
}

impl Drop for MonitorController {
    fn drop(&mut self) {
        if let Some(sender) = self.poll_sender.take() {
            let _ = sender.try_send(PollCommand::Stop);
        }
    }
}
