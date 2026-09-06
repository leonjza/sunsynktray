use crate::ui::shell::StatusBar;
use gpui_kit::component::input::InputState;
use gpui_kit::*;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

mod controller;
mod history;
mod polling;
mod session;
mod state;
mod view;
mod window;
pub(crate) use controller::{MonitorController, MonitorControllerGlobal};
pub(crate) use state::{
    ConnectionState, HistoryPointIndex, HistorySnapshot, MonitorState, MonitorStateGlobal, Screen,
    TrayMetric,
};
pub(crate) use window::open_main_window;

pub(crate) struct Dashboard {
    state: Arc<MonitorState>,
    controller: Entity<MonitorController>,
    screen: Screen,
    email: Entity<InputState>,
    password: Entity<InputState>,
    refresh_interval: Entity<InputState>,
    hovered_history: Option<usize>,
    status_bar: Entity<StatusBar>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    startup_enabled: bool,
    startup_pending: bool,
    startup_generation: u64,
    startup_error: Option<String>,
    refresh_interval_error: Option<String>,
    credentials_synced: bool,
}
impl Dashboard {
    pub(crate) fn new(
        state: Arc<MonitorState>,
        controller: Entity<MonitorController>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let email = cx.new(|cx| InputState::new(window, cx).placeholder("you@example.com"));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("SunSynk password")
                .masked(true)
        });
        let refresh_interval = cx.new(|cx| InputState::new(window, cx).default_value("60"));
        let status_bar = cx.new(|_| StatusBar::new());
        let status_entity = status_bar.downgrade();
        cx.spawn(async move |_, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
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
            controller: controller.clone(),
            screen: Screen::Dashboard,
            email,
            password,
            refresh_interval,
            hovered_history: None,
            status_bar,
            chart_bounds: Arc::new(Mutex::new(None)),
            startup_enabled: false,
            startup_pending: true,
            startup_error: None,
            startup_generation: 0,
            refresh_interval_error: None,
            credentials_synced: false,
        };
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(async { crate::platform::startup::is_enabled() })
                .await;
            entity.update(cx, |dashboard, cx| {
                if dashboard.startup_generation != 0 {
                    return;
                }
                dashboard.startup_pending = false;
                match result {
                    Ok(enabled) => dashboard.startup_enabled = enabled,
                    Err(error) => {
                        tracing::warn!(%error, "could not read startup setting");
                        dashboard.startup_error = Some(error.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.observe(&controller, |_, _, cx| cx.notify()).detach();
        dashboard
    }

    pub(crate) fn set_startup_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.startup_pending {
            return;
        }
        self.startup_generation = self.startup_generation.wrapping_add(1);
        let generation = self.startup_generation;
        self.startup_pending = true;
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { crate::platform::startup::set_enabled(enabled) })
                .await;
            entity.update(cx, |dashboard, cx| {
                if dashboard.startup_generation != generation {
                    return;
                }
                dashboard.startup_pending = false;
                match result {
                    Ok(()) => {
                        dashboard.startup_enabled = enabled;
                        dashboard.startup_error = None;
                    }
                    Err(error) => {
                        dashboard.startup_error = Some(error.to_string());
                        tracing::warn!(%error, "could not update startup setting");
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn set_refresh_interval_error(
        &mut self,
        error: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.refresh_interval_error = error;
        cx.notify();
    }
    pub(crate) fn set_tray_metric(&mut self, metric: Option<TrayMetric>, cx: &mut Context<Self>) {
        self.controller
            .update(cx, |controller, cx| controller.set_tray_metric(metric, cx));
        cx.notify();
    }

    pub(crate) fn reconnect_or_connect(
        &mut self,
        email: String,
        password: String,
        refresh_seconds: u64,
        cx: &mut Context<Self>,
    ) {
        self.refresh_interval_error = None;
        self.controller.update(cx, |controller, cx| {
            controller.reconnect_or_connect(email, password, refresh_seconds, cx)
        });
    }

    pub(crate) fn refresh_now(&mut self, cx: &mut Context<Self>) {
        self.controller
            .update(cx, |controller, cx| controller.refresh_now(cx));
    }

    pub(crate) fn select_inverter(&mut self, serial: String, cx: &mut Context<Self>) {
        self.controller
            .update(cx, |controller, cx| controller.select_inverter(serial, cx));
        self.screen = Screen::Dashboard;
        cx.notify();
    }

    pub(crate) fn open_dashboard(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Dashboard;
        cx.notify();
    }

    pub(crate) fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Settings;
        cx.notify();
    }
}
