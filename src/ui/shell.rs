use crate::app::{Dashboard, MonitorController, Screen};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    spinner::Spinner,
    status_bar::StatusBar as KitStatusBar,
    Icon, IconName, Sizable, StyledExt, Theme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::sync::{Arc, Mutex};

pub(crate) struct StatusBar {
    controller: Entity<MonitorController>,
    connection_log_window: Arc<Mutex<Option<AnyWindowHandle>>>,
    screen: Screen,
    activity: String,
    fetching: bool,
    next_refresh_in: Option<u64>,
    source_activity: String,
    source_fetching: bool,
    source_next_refresh_in: Option<u64>,
    source_refresh_generation: u64,
}

impl Render for StatusBar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        render_status_bar(
            self.screen,
            &self.activity,
            self.fetching,
            self.next_refresh_in,
            self.controller.clone(),
            self.connection_log_window.clone(),
        )
    }
}

impl StatusBar {
    pub(crate) fn new(controller: Entity<MonitorController>) -> Self {
        Self {
            controller,
            connection_log_window: Arc::new(Mutex::new(None)),
            screen: Screen::Dashboard,
            activity: "Starting…".into(),
            fetching: false,
            next_refresh_in: None,
            source_activity: String::new(),
            source_fetching: false,
            source_next_refresh_in: None,
            source_refresh_generation: 0,
        }
    }

    pub(crate) fn sync(
        &mut self,
        screen: Screen,
        activity: String,
        fetching: bool,
        next_refresh_in: Option<u64>,
        refresh_generation: u64,
        cx: &mut Context<Self>,
    ) {
        if self.screen == screen
            && self.source_activity == activity
            && self.source_fetching == fetching
            && self.source_next_refresh_in == next_refresh_in
            && self.source_refresh_generation == refresh_generation
        {
            return;
        }

        self.screen = screen;
        self.activity = activity.clone();
        self.fetching = fetching;
        self.next_refresh_in = next_refresh_in;
        self.source_activity = activity;
        self.source_fetching = fetching;
        self.source_next_refresh_in = next_refresh_in;
        self.source_refresh_generation = refresh_generation;
        cx.notify();
    }

}

pub(crate) fn toolbar(
    theme: &Theme,
    screen: Screen,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    div()
        .h_flex()
        .h(px(48.))
        .flex_shrink_0()
        .px_4()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
        .child(Icon::new(IconName::Sun).size_4())
        .child(div().font_weight(FontWeight::SEMIBOLD).child("SunTray"))
        .child(div().flex_1())
        .child(
            Button::new("dashboard")
                .label("Dashboard")
                .when(screen == Screen::Dashboard, |b| b.primary())
                .when(screen != Screen::Dashboard, |b| b.ghost())
                .xsmall()
                .h(px(28.))
                .on_click({
                    let entity = entity.clone();
                    move |_, _, cx| entity.update(cx, |dashboard, cx| dashboard.open_dashboard(cx))
                }),
        )
        .child(
            Button::new("settings")
                .label("Settings")
                .when(screen == Screen::Settings, |b| b.primary())
                .when(screen != Screen::Settings, |b| b.ghost())
                .xsmall()
                .h(px(28.))
                .on_click(move |_, _, cx| {
                    entity.update(cx, |dashboard, cx| dashboard.open_settings(cx))
                }),
        )
}

fn render_status_bar(
    screen: Screen,
    activity: &str,
    fetching: bool,
    next_refresh_in: Option<u64>,
    controller: Entity<MonitorController>,
    connection_log_window: Arc<Mutex<Option<AnyWindowHandle>>>,
) -> impl IntoElement {
    let show_activity = screen == Screen::Dashboard
        || fetching
        || activity.starts_with("Login failed")
        || activity.starts_with("Refresh failed")
        || activity.starts_with("History unavailable")
        || activity.starts_with("Polling stopped");
    let activity = if show_activity {
        refresh_activity(activity, next_refresh_in)
    } else {
        "Settings".to_owned()
    };
    let activity_content = div()
        .h_flex()
        .items_center()
        .gap_2()
        .child(if fetching {
            div()
                .h_flex()
                .items_center()
                .justify_center()
                .size_4()
                .child(
                    Spinner::new()
                        .small()
                        .color(rgb(0x34c759).into()),
                )
                .into_any_element()
        } else {
            Icon::new(IconName::Globe)
                .size_4()
                .into_any_element()
        })
        .child(activity)
        .into_any_element();
    let connection_log = Button::new("connection-log")
        .icon(IconName::FileText)
        .accessibility_label("Open connection log")
        .tooltip("Connection log")
        .ghost()
        .xsmall()
        .on_click(move |_, _, cx| {
            crate::app::open_connection_log_window(
                cx,
                controller.clone(),
                connection_log_window.clone(),
            );
        });
    KitStatusBar::new()
        .left(activity_content)
        .right(connection_log)
        .right(format!("v{}", env!("CARGO_PKG_VERSION")))
        .into_any_element()
}

fn refresh_activity(activity: &str, next_refresh_in: Option<u64>) -> String {
    match next_refresh_in {
        Some(seconds) if activity.starts_with("Refresh failed") => {
            format!("Refresh failed · retry in {seconds}s")
        }
        Some(seconds) if activity.starts_with("Waiting for next refresh") => {
            format!("Waiting for next refresh · next refresh in {seconds}s")
        }
        _ => activity.to_owned(),
    }
}
