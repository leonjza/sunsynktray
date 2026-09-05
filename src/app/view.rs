use crate::{
    app::{Dashboard, Screen},
    ui::{
        dashboard as dashboard_view,
        settings::{self as settings_view, SettingsView},
        shell,
    },
};
use gpui_kit::component::{ActiveTheme, StyledExt};
use gpui_kit::prelude::InteractiveElement;
use gpui_kit::*;

impl Render for Dashboard {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (status_activity, status_fetching, status_next_refresh_in) = {
            let controller = self.controller.read(cx);
            (
                controller.activity.clone(),
                controller.fetching,
                controller.next_refresh_in,
            )
        };
        let status_bar = self.status_bar.clone();
        let status_screen = self.screen;
        status_bar.update(cx, |status, cx| {
            status.sync(
                status_screen,
                status_activity,
                status_fetching,
                status_next_refresh_in,
                cx,
            );
        });
        let controller = self.controller.read(cx);
        let theme = cx.theme();
        let entity = cx.entity().clone();
        let chart_bounds = self.chart_bounds.clone();
        let hover_entity = entity.clone();
        let mut root = div()
            .v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground);
        root = root.on_mouse_move(move |event, _, cx| {
            let inside_chart = chart_bounds
                .lock()
                .ok()
                .and_then(|bounds| {
                    bounds
                        .as_ref()
                        .map(|bounds| bounds.contains(&event.position))
                })
                .unwrap_or(false);
            if !inside_chart {
                hover_entity.update(cx, |dashboard, cx| dashboard.hover_history(None, cx));
            }
        });
        root.child(gpui_kit::component::TitleBar::new())
            .child(shell::toolbar(theme, self.screen, entity.clone()))
            .child(match self.screen {
                Screen::Dashboard => dashboard_view::render(
                    theme,
                    &self.state,
                    &controller.connection,
                    status_fetching,
                    controller.history_date,
                    self.hovered_history,
                    self.chart_bounds.clone(),
                    controller.inverters.iter().find(|inverter| {
                        Some(&inverter.serial) == controller.selected_serial.as_ref()
                    }),
                    entity.clone(),
                ),
                Screen::Settings => settings_view::render(SettingsView {
                    theme,
                    email: &self.email,
                    password: &self.password,
                    refresh_interval: &self.refresh_interval,
                    connection: &controller.connection,
                    inverters: &controller.inverters,
                    selected: &controller.selected_serial,
                    tray_metric: controller.tray_metric,
                    fetching: status_fetching,
                    startup_enabled: self.startup_enabled,
                    startup_error: self.startup_error.clone(),
                    entity,
                }),
            })
            .child(self.status_bar.clone())
    }
}
