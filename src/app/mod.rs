use crate::{storage::credentials, ui::shell::StatusBar};
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
pub(crate) use state::{ConnectionState, MonitorState, MonitorStateGlobal, Screen, TrayMetric};
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
    startup_error: Option<String>,
}
impl Dashboard {
    pub(crate) fn new(
        state: Arc<MonitorState>,
        controller: Entity<MonitorController>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let saved = match credentials::load() {
            Ok(saved) => saved,
            Err(error) => {
                tracing::warn!(%error, "could not read saved SunSynk credentials");
                None
            }
        };
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
            cx.background_executor().timer(Duration::from_secs(1)).await;
            status_entity.update(cx, |status, cx| {
                status.tick_countdown(cx);
            });
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
            startup_enabled: crate::platform::startup::is_enabled().unwrap_or_else(|error| {
                tracing::warn!(%error, "could not read startup setting");
                false
            }),
            startup_error: None,
        };
        cx.observe(&controller, |_, _, cx| cx.notify()).detach();
        dashboard
    }

    pub(crate) fn set_startup_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        match crate::platform::startup::set_enabled(enabled) {
            Ok(()) => {
                self.startup_enabled = enabled;
                self.startup_error = None;
            }
            Err(error) => {
                self.startup_error = Some(error.to_string());
                tracing::warn!(%error, "could not update startup setting");
            }
        }
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
