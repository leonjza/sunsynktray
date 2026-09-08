use crate::ui::shell::StatusBar;
use chrono::Datelike;
use futures_util::future::{select, Either};
use gpui_kit::component::{
    calendar::Date,
    date_picker::{DatePickerEvent, DatePickerState},
    input::InputState,
};
use gpui_kit::*;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

mod controller;
mod history;
mod polling;
mod runtime;
mod session;
mod state;
mod view;
mod window;
pub(crate) use controller::{MonitorController, MonitorControllerGlobal};
pub(crate) use runtime::shutdown as shutdown_runtime;
pub(crate) use state::{
    ConnectionState, HistoryPointIndex, HistorySnapshot, HistorySource, MonitorState,
    MonitorStateGlobal, Screen, TrayMetric,
};
pub(crate) use window::{open_connection_log_window, open_main_window};

pub(crate) struct Dashboard {
    state: Arc<MonitorState>,
    controller: Entity<MonitorController>,
    screen: Screen,
    email: Entity<InputState>,
    password: Entity<InputState>,
    refresh_interval: Entity<InputState>,
    history_days: Entity<InputState>,
    hovered_history: Option<usize>,
    status_bar: Entity<StatusBar>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    startup_enabled: bool,
    startup_pending: bool,
    startup_generation: u64,
    startup_error: Option<String>,
    refresh_interval_error: Option<String>,
    credentials_synced: bool,
    history_days_generation: Arc<AtomicU64>,
    history_date_picker: Entity<DatePickerState>,
    _subscriptions: Vec<Subscription>,
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
        let history_days = cx.new(|cx| InputState::new(window, cx).default_value("365"));
        let history_date = controller.read(cx).history_date;
        let history_date_picker = cx.new(|cx| {
            let mut picker = DatePickerState::new(window, cx)
                .date_format("%Y-%m-%d")
                .disabled_matcher(|date: &chrono::NaiveDate| {
                    *date > chrono::Local::now().date_naive()
                });
            picker.set_date(Date::Single(Some(history_date)), window, cx);
            picker.set_year_range((history_date.year() - 10, history_date.year() + 1), cx);
            picker
        });
        let history_days_observed = history_days.clone();
        let history_days_generation = Arc::new(AtomicU64::new(0));
        let status_bar = cx.new(|_| StatusBar::new(controller.clone()));
        let status_entity = status_bar.downgrade();
        let mut dashboard = Self {
            state,
            controller: controller.clone(),
            screen: Screen::Dashboard,
            email,
            password,
            refresh_interval,
            history_days,
            hovered_history: None,
            status_bar,
            chart_bounds: Arc::new(Mutex::new(None)),
            startup_enabled: false,
            startup_pending: true,
            startup_error: None,
            startup_generation: 0,
            refresh_interval_error: None,
            credentials_synced: false,
            history_days_generation: history_days_generation.clone(),
            history_date_picker: history_date_picker.clone(),
            _subscriptions: Vec::new(),
        };
        let date_picker_subscription = cx.subscribe(
            &history_date_picker,
            |dashboard, _, event: &DatePickerEvent, cx| {
                if let DatePickerEvent::Change(Date::Single(Some(date))) = event {
                    dashboard.select_history_date(*date, cx);
                }
            },
        );
        dashboard._subscriptions.push(date_picker_subscription);
        let dashboard_timer = cx.entity().downgrade();
        cx.spawn(async move |_, cx| loop {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            let Ok((settings, controller)) = dashboard_timer.update(cx, |dashboard, _| {
                (
                    dashboard.screen == Screen::Settings,
                    dashboard.controller.clone(),
                )
            }) else {
                break;
            };
            let changed =
                controller.update(cx, |controller, _| controller.tick_refresh_countdown());
            if settings && changed && dashboard_timer.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
            if status_entity
                .update(cx, |status, cx| status.tick_countdown(cx))
                .is_err()
            {
                break;
            }
        })
        .detach();
        let dashboard_entity = cx.entity().clone();
        let history_days_debounce = Arc::new(Mutex::new(None));
        cx.observe(&history_days_observed, move |_, _, cx| {
            let generation = history_days_generation.fetch_add(1, Ordering::AcqRel) + 1;
            let (cancel_sender, cancel_receiver) = tokio::sync::oneshot::channel();
            let previous_cancel = history_days_debounce
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .replace(cancel_sender);
            if let Some(cancel) = previous_cancel {
                let _ = cancel.send(());
            }
            let dashboard_entity = dashboard_entity.clone();
            let history_days_debounce = history_days_debounce.clone();
            let history_days_generation = history_days_generation.clone();
            cx.spawn(async move |_, cx| {
                let timer = cx.background_executor().timer(Duration::from_millis(350));
                if matches!(
                    select(Box::pin(timer), Box::pin(cancel_receiver)).await,
                    Either::Left(_)
                ) {
                    if history_days_generation.load(Ordering::Acquire) != generation {
                        return;
                    }
                    history_days_debounce
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .take();
                    let reconnect = dashboard_entity.update(cx, |dashboard, cx| {
                        dashboard.history_days_edit(generation, cx)
                    });
                    if let Some((controller, email, password, refresh_seconds, history_days)) =
                        reconnect
                    {
                        controller.update(cx, |controller, cx| {
                            controller.reconnect_or_connect(
                                email,
                                password,
                                refresh_seconds,
                                history_days,
                                cx,
                            );
                        });
                    }
                }
            })
            .detach();
        })
        .detach();
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
        let history_date_picker = history_date_picker.clone();
        cx.observe_in(
            &controller,
            window,
            move |dashboard, controller, window, cx| {
                let date = controller.read(cx).history_date;
                let picker_date = dashboard.history_date_picker.read(cx).date();
                if picker_date != Date::Single(Some(date)) {
                    history_date_picker.update(cx, |picker, cx| {
                        picker.set_date(Date::Single(Some(date)), window, cx);
                    });
                }
                cx.notify();
            },
        )
        .detach();
        dashboard
    }

    fn history_days_edit(
        &mut self,
        generation: u64,
        cx: &mut Context<Self>,
    ) -> Option<(Entity<MonitorController>, String, String, u64, u64)> {
        if generation != self.history_days_generation.load(Ordering::Acquire) {
            return None;
        }
        let Ok(history_days) = self.history_days.read(cx).value().parse::<u64>() else {
            return None;
        };
        if !(1..=3650).contains(&history_days) {
            return None;
        }
        let (email, password) = self.controller.read(cx).credentials.clone()?;
        let controller = self.controller.clone();
        let refresh_seconds = controller.read(cx).refresh_seconds;
        if controller.read(cx).history_days == history_days {
            return None;
        }
        Some((controller, email, password, refresh_seconds, history_days))
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

    pub(crate) fn reconnect(
        &mut self,
        email: String,
        password: String,
        refresh_seconds: u64,
        history_days: u64,
        cx: &mut Context<Self>,
    ) {
        self.refresh_interval_error = None;
        self.controller.update(cx, |controller, cx| {
            controller.reconnect(email, password, refresh_seconds, history_days, cx)
        });
    }

    pub(crate) fn refresh_now(&mut self, cx: &mut Context<Self>) {
        self.controller
            .update(cx, |controller, cx| controller.refresh_now(cx));
    }

    pub(crate) fn toggle_backfill(&mut self, cx: &mut Context<Self>) {
        self.controller
            .update(cx, |controller, cx| controller.toggle_backfill(cx));
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
