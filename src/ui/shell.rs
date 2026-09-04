use crate::app::{Dashboard, Screen};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    ActiveTheme, Icon, IconName, Sizable, StyledExt, Theme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(crate) struct StatusBar {
    screen: Screen,
    activity: String,
    fetching: bool,
    next_refresh_in: Option<u64>,
    source_activity: String,
    source_fetching: bool,
    source_next_refresh_in: Option<u64>,
}

impl Render for StatusBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        render_status_bar(cx.theme(), self.screen, &self.activity, self.fetching)
    }
}

impl StatusBar {
    pub(crate) fn new() -> Self {
        Self {
            screen: Screen::Dashboard,
            activity: "Starting…".into(),
            fetching: false,
            next_refresh_in: None,
            source_activity: String::new(),
            source_fetching: false,
            source_next_refresh_in: None,
        }
    }

    pub(crate) fn sync(
        &mut self,
        screen: Screen,
        activity: String,
        fetching: bool,
        next_refresh_in: Option<u64>,
        cx: &mut Context<Self>,
    ) {
        if self.screen == screen
            && self.source_activity == activity
            && self.source_fetching == fetching
            && self.source_next_refresh_in == next_refresh_in
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
        cx.notify();
    }

    pub(crate) fn tick_countdown(&mut self, cx: &mut Context<Self>) {
        let Some(seconds) = self.next_refresh_in.as_mut() else {
            return;
        };
        if *seconds == 0 {
            return;
        }
        *seconds = seconds.saturating_sub(1);
        self.activity = if self.activity.starts_with("Refresh failed") {
            format!("Refresh failed · retry in {}s", *seconds)
        } else {
            format!("Waiting for next refresh · next refresh in {}s", *seconds)
        };
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
    theme: &Theme,
    screen: Screen,
    activity: &str,
    fetching: bool,
) -> impl IntoElement {
    let show_activity = screen == Screen::Dashboard
        || fetching
        || activity.starts_with("Login failed")
        || activity.starts_with("Refresh failed")
        || activity.starts_with("History unavailable")
        || activity.starts_with("Polling stopped");
    div()
        .h_flex()
        .h(px(28.))
        .px_4()
        .items_center()
        .border_t_1()
        .border_color(theme.border)
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(if show_activity {
            activity.to_owned()
        } else {
            "Settings".to_owned()
        })
        .child(div().flex_1())
        .child(format!("v{}", env!("CARGO_PKG_VERSION")))
}
