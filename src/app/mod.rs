use crate::{
    app::polling::protocol::Command as PollCommand, domain::InverterSummary, platform::tray,
    storage::credentials, ui::shell::StatusBar,
};
use gpui::Timer;
use gpui::*;
use gpui_component::input::InputState;
use std::{
    sync::{atomic::AtomicU64, Arc, Mutex},
    time::Duration,
};

mod history;
mod polling;
mod session;
mod state;
mod view;
mod window;
pub(crate) use state::{ConnectionState, MonitorState, MonitorStateGlobal, Screen, TrayMetric};
pub(crate) use window::open_main_window;

pub(crate) struct Dashboard {
    state: Arc<MonitorState>,
    screen: Screen,
    email: Entity<InputState>,
    password: Entity<InputState>,
    refresh_interval: Entity<InputState>,
    refresh_seconds: u64,
    tray_metric: Option<TrayMetric>,
    next_refresh_in: Option<u64>,
    activity: String,
    fetching: bool,
    polling: bool,
    poll_generation: u64,
    connect_generation: u64,
    connect_epoch: Arc<AtomicU64>,
    poll_sender: Option<tokio::sync::mpsc::Sender<PollCommand>>,
    connection: ConnectionState,
    inverters: Vec<InverterSummary>,
    selected_serial: Option<String>,
    credentials: Option<(String, String)>,
    refresh_token: Option<String>,
    history_date: chrono::NaiveDate,
    history_is_manual: bool,
    history_previous_date: Option<chrono::NaiveDate>,
    hovered_history: Option<usize>,
    has_cached_data: bool,
    status_bar: Entity<StatusBar>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
}
impl Dashboard {
    pub(crate) fn new(
        state: Arc<MonitorState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let has_cached_data = state.has_live_data();
        let saved = match credentials::load() {
            Ok(saved) => saved,
            Err(error) => {
                tracing::warn!(%error, "could not read saved SunSynk credentials");
                None
            }
        };
        let startup_credentials = saved.clone().map(|saved| (saved.email, saved.password));
        let email = cx.new(|cx| {
            let input = InputState::new(window, cx).placeholder("you@example.com");
            if let Some(saved) = &saved {
                input.default_value(saved.email.clone())
            } else {
                input
            }
        });
        let password = cx.new(|cx| {
            let input = InputState::new(window, cx)
                .placeholder("SunSynk password")
                .masked(true);
            if let Some(saved) = &saved {
                input.default_value(saved.password.clone())
            } else {
                input
            }
        });
        let default_refresh_seconds = saved
            .as_ref()
            .and_then(|saved| saved.refresh_seconds)
            .unwrap_or(60)
            .clamp(1, 3600);
        let refresh_interval = cx.new(|cx| {
            InputState::new(window, cx).default_value(default_refresh_seconds.to_string())
        });
        let status_bar = cx.new(|_| StatusBar::new());
        let status_entity = status_bar.clone();
        cx.spawn(async move |_, cx| loop {
            Timer::after(Duration::from_secs(1)).await;
            if status_entity
                .update(cx, |status, cx| {
                    status.tick_countdown(cx);
                })
                .is_err()
            {
                break;
            }
        })
        .detach();
        let dashboard = Self {
            state,
            screen: Screen::Dashboard,
            email,
            password,
            refresh_interval,
            refresh_seconds: default_refresh_seconds,
            tray_metric: saved
                .as_ref()
                .and_then(|saved| TrayMetric::from_saved(saved.tray_metric.as_deref())),
            next_refresh_in: None,
            activity: if startup_credentials.is_some() {
                if has_cached_data {
                    "Reconnecting…".into()
                } else {
                    "Starting…".into()
                }
            } else {
                "No account configured".into()
            },
            polling: false,
            poll_generation: 0,
            connect_generation: 0,
            connect_epoch: Arc::new(AtomicU64::new(0)),
            fetching: false,
            poll_sender: None,
            connection: if startup_credentials.is_some() {
                if has_cached_data {
                    ConnectionState::Connected
                } else {
                    ConnectionState::Connecting
                }
            } else {
                ConnectionState::Unconfigured
            },
            inverters: Vec::new(),
            selected_serial: saved
                .as_ref()
                .and_then(|saved| saved.selected_serial.clone()),
            credentials: startup_credentials.clone(),
            refresh_token: saved.as_ref().and_then(|saved| saved.refresh_token.clone()),
            history_date: chrono::Local::now().date_naive(),
            history_is_manual: false,
            history_previous_date: None,
            hovered_history: None,
            has_cached_data,
            status_bar,
            chart_bounds: Arc::new(Mutex::new(None)),
        };
        if let Some((email, password)) = startup_credentials {
            let entity = cx.entity().clone();
            cx.spawn(async move |_, cx| {
                Timer::after(Duration::from_millis(10)).await;
                entity
                    .update(cx, |dashboard, cx| dashboard.connect(email, password, cx))
                    .ok();
            })
            .detach();
        }
        dashboard
    }
    pub(crate) fn set_tray_metric(&mut self, metric: Option<TrayMetric>, cx: &mut Context<Self>) {
        self.tray_metric = metric;
        credentials::save_tray_metric_async(metric.map(TrayMetric::saved_name).map(str::to_owned));
        self.update_tray(cx);
        cx.notify();
    }

    fn update_tray(&self, cx: &mut Context<Self>) {
        let snapshot = self.state.snapshot();
        let connected = matches!(
            self.connection,
            ConnectionState::Connected | ConnectionState::Stale
        );
        let value = connected
            .then(|| self.tray_metric.map(|metric| metric.value(&snapshot)))
            .flatten();
        let symbol = if !connected {
            "bolt.slash.fill"
        } else {
            match self.tray_metric {
                Some(TrayMetric::Soc) => "battery.100",
                Some(TrayMetric::Load) => "house.fill",
                Some(TrayMetric::Solar) => "sun.max.fill",
                None => "bolt.fill",
            }
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
        tray::update(cx, value.as_deref(), symbol, &tooltip);
    }

    pub(crate) fn open_dashboard(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Dashboard;
        cx.notify();
    }

    pub(crate) fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Settings;
        cx.notify();
    }
    fn send_poll_command(&mut self, command: PollCommand, cx: &mut Context<Self>) {
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

impl Drop for Dashboard {
    fn drop(&mut self) {
        if let Some(sender) = self.poll_sender.take() {
            let _ = sender.try_send(PollCommand::Stop);
        }
    }
}
