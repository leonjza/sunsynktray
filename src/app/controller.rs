use crate::{
    app::polling::protocol::Command as PollCommand, domain::InverterSummary, storage::credentials,
};
use gpui_kit::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use super::{ConnectionState, HistorySource, MonitorState, TrayMetric};

pub(crate) struct MonitorController {
    pub(crate) state: Arc<MonitorState>,
    pub(crate) fetching: bool,
    pub(crate) polling: bool,
    pub(crate) poll_generation: u64,
    pub(crate) connect_generation: u64,
    pub(crate) connect_epoch: Arc<AtomicU64>,
    pub(crate) connect_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    pub(crate) connect_task: Option<tokio::task::JoinHandle<()>>,
    pub(crate) poll_sender: Option<tokio::sync::mpsc::Sender<PollCommand>>,
    pub(crate) poll_cancel: Option<tokio::sync::oneshot::Sender<()>>,
    pub(crate) poll_task: Option<tokio::task::JoinHandle<()>>,
    pub(crate) connection: ConnectionState,
    pub(crate) inverters: Vec<InverterSummary>,
    pub(crate) selected_serial: Option<String>,
    pub(crate) credentials: Option<(String, String)>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) auth_state: Option<std::sync::Arc<std::sync::Mutex<crate::sunsynk::AuthState>>>,
    pub(crate) refresh_seconds: u64,
    pub(crate) history_days: u64,
    pub(crate) backfill_completed: u64,
    pub(crate) backfill_total: u64,
    pub(crate) backfill_running: bool,
    pub(crate) backfill_detail: String,
    pub(crate) backfill_next_request_in: Option<u64>,
    pub(crate) activity: String,
    pub(crate) connection_log: Arc<Mutex<VecDeque<String>>>,
    pub(crate) connection_log_revision: Arc<AtomicU64>,
    pub(crate) connection_log_signal: Arc<tokio::sync::Notify>,
    pub(crate) next_refresh_in: Option<u64>,
    pub(crate) refresh_generation: u64,
    pub(crate) has_cached_data: bool,
    pub(crate) tray_metric: Option<TrayMetric>,
    pub(crate) always_on_top: bool,
    pub(crate) compact_view: bool,
    pub(crate) history_date: chrono::NaiveDate,
    pub(crate) history_source: HistorySource,
    pub(crate) history_cache:
        HashMap<(String, chrono::NaiveDate), (Vec<crate::domain::HistorySeries>, HistorySource)>,
    pub(crate) history_cache_order: VecDeque<(String, chrono::NaiveDate)>,
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
            connect_task: None,
            poll_sender: None,
            poll_cancel: None,
            poll_task: None,
            connection: ConnectionState::Unconfigured,
            inverters: Vec::new(),
            selected_serial: None,
            credentials: None,
            refresh_token: None,
            auth_state: None,
            refresh_seconds: 60,
            history_days: 365,
            backfill_completed: 0,
            backfill_total: 0,
            backfill_running: false,
            backfill_detail: String::new(),
            backfill_next_request_in: None,
            activity: if has_cached_data {
                "Starting…"
            } else {
                "Loading saved account…"
            }
            .into(),
            connection_log: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            connection_log_revision: Arc::new(AtomicU64::new(0)),
            connection_log_signal: Arc::new(tokio::sync::Notify::new()),
            next_refresh_in: None,
            refresh_generation: 0,
            has_cached_data,
            tray_metric: None,
            always_on_top: false,
            compact_view: false,
            history_date: chrono::Local::now().date_naive(),
            history_source: HistorySource::Cached,
            history_cache: HashMap::new(),
            history_cache_order: VecDeque::new(),
            history_is_manual: false,
            history_previous_date: None,
            credentials_loaded: false,
        }
    }

    pub(crate) fn record_connection_event(&mut self, message: impl Into<String>) {
        Self::record_connection_event_to(
            &self.connection_log,
            &self.connection_log_revision,
            &self.connection_log_signal,
            message,
        );
    }

    pub(crate) fn record_connection_event_to(
        connection_log: &Arc<Mutex<VecDeque<String>>>,
        connection_log_revision: &Arc<AtomicU64>,
        connection_log_signal: &Arc<tokio::sync::Notify>,
        message: impl Into<String>,
    ) {
        const MAX_CONNECTION_LOG_ENTRIES: usize = 1000;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut connection_log = connection_log
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        connection_log.push_back(format!("[{timestamp}] {}", message.into()));
        if connection_log.len() > MAX_CONNECTION_LOG_ENTRIES {
            connection_log.pop_front();
        }
        connection_log_revision.fetch_add(1, Ordering::Release);
        connection_log_signal.notify_one();
    }

    pub(crate) fn cache_history(
        &mut self,
        serial: String,
        date: chrono::NaiveDate,
        history: Vec<crate::domain::HistorySeries>,
        source: HistorySource,
    ) {
        const MAX_HISTORY_CACHE_ENTRIES: usize = 128;
        let key = (serial, date);
        self.history_cache.insert(key.clone(), (history, source));
        self.history_cache_order.retain(|cached| cached != &key);
        self.history_cache_order.push_back(key);
        while self.history_cache_order.len() > MAX_HISTORY_CACHE_ENTRIES {
            if let Some(oldest) = self.history_cache_order.pop_front() {
                self.history_cache.remove(&oldest);
            }
        }
    }

    pub(crate) fn tick_refresh_countdown(&mut self) -> bool {
        let mut changed = false;
        if let Some(seconds) = self.next_refresh_in.as_mut() {
            if *seconds > 0 {
                *seconds = seconds.saturating_sub(1);
                changed = true;
            }
        }
        if let Some(seconds) = self.backfill_next_request_in.as_mut() {
            if *seconds > 1 {
                *seconds = seconds.saturating_sub(1);
                changed = true;
            } else if *seconds == 1 {
                *seconds = 0;
                changed = true;
            }
        }
        changed
    }

    pub(crate) fn initialize(&mut self, cx: &mut Context<Self>) {
        let entity = cx.entity().clone();
        let database = self.state.database.clone();
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let saved = credentials::load()?;
                    let preferences = crate::storage::settings::load()?.unwrap_or_default();
                    let selected_serial = preferences.selected_serial.clone();
                    let cached = saved.as_ref().map(|saved| {
                        (
                            database.load_snapshot(saved.email.clone()).ok().flatten(),
                            database
                                .load_history(
                                    saved.email.clone(),
                                    selected_serial.clone(),
                                    chrono::Local::now().date_naive().to_string(),
                                )
                                .unwrap_or_default(),
                        )
                    });
                    Ok::<_, anyhow::Error>((saved, cached, preferences, selected_serial))
                })
                .await;
            entity.update(cx, |controller, cx| {
                controller.credentials_loaded = true;
                match result {
                    Ok((
                        Some(saved),
                        Some((cached_snapshot, cached_history)),
                        preferences,
                        selected_serial,
                    )) => {
                        let email = saved.email.clone();
                        let password = saved.password.clone();
                        let cached_history_for_cache = cached_history.clone();
                        if let Some(snapshot) = cached_snapshot {
                            controller.state.set_cached_data(snapshot, cached_history);
                            controller.has_cached_data = true;
                        } else if !cached_history.is_empty() {
                            controller.state.set_history(cached_history);
                        }
                        if let Some(serial) = selected_serial.clone() {
                            if !cached_history_for_cache.is_empty() {
                                controller.cache_history(
                                    serial,
                                    controller.history_date,
                                    cached_history_for_cache,
                                    HistorySource::Cached,
                                );
                            }
                        }
                        controller.connection = if controller.has_cached_data {
                            ConnectionState::Stale
                        } else {
                            ConnectionState::Connecting
                        };
                        controller.selected_serial = selected_serial;
                        controller.credentials = Some((email.clone(), password.clone()));
                        controller.refresh_token = saved.refresh_token;
                        controller.refresh_seconds = preferences.refresh_seconds.clamp(1, 3600);
                        controller.history_days = preferences.history_days.clamp(1, 3650);
                        controller.tray_metric =
                            TrayMetric::from_saved(preferences.tray_metric.as_deref());
                        controller.always_on_top = preferences.always_on_top;
                        controller.compact_view = preferences.compact_view;
                        controller.activity = if controller.has_cached_data {
                            "Reconnecting…"
                        } else {
                            "Starting…"
                        }
                        .into();
                        controller.connect(email, password, cx);
                    }
                    Ok((Some(saved), None, preferences, selected_serial)) => {
                        let email = saved.email.clone();
                        let password = saved.password.clone();
                        controller.connection = ConnectionState::Connecting;
                        controller.credentials = Some((email.clone(), password.clone()));
                        controller.refresh_token = saved.refresh_token;
                        controller.selected_serial = selected_serial;
                        controller.refresh_seconds = preferences.refresh_seconds.clamp(1, 3600);
                        controller.history_days = preferences.history_days.clamp(1, 3650);
                        controller.tray_metric =
                            TrayMetric::from_saved(preferences.tray_metric.as_deref());
                        controller.always_on_top = preferences.always_on_top;
                        controller.compact_view = preferences.compact_view;
                        controller.connect(email, password, cx);
                    }
                    Ok((None, _, _, _)) => {
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
        crate::storage::settings::save_tray_metric_async(
            metric.map(TrayMetric::saved_name).map(str::to_owned),
        );
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
            (true, Some(TrayMetric::Soc)) => battery_symbol(snapshot.battery_soc),
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

fn battery_symbol(soc: f64) -> &'static str {
    match soc.clamp(0.0, 100.0) {
        soc if soc < 12.5 => "battery.0percent",
        soc if soc < 37.5 => "battery.25percent",
        soc if soc < 62.5 => "battery.50percent",
        soc if soc < 87.5 => "battery.75percent",
        _ => "battery.100percent",
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
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        if let Some(cancel) = self.connect_cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(task) = self.connect_task.take() {
            task.abort();
        }
    }
}
