use crate::{
    app::{Dashboard, TrayMetric},
    domain::InverterSummary,
};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    scroll::ScrollableElement,
    switch::Switch,
    Disableable, Sizable, StyledExt, Theme,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(crate) fn render(view: SettingsView<'_>) -> AnyElement {
    let SettingsView {
        theme,
        email,
        password,
        refresh_interval,
        connection,
        inverters,
        selected,
        tray_metric,
        fetching,
        startup_enabled,
        startup_error,
        entity,
    } = view;
    div()
        .v_flex()
        .flex_1()
        .overflow_y_scrollbar()
        .p_3()
        .gap_2()
        .child(
            div()
                .v_flex()
                .gap_0()
                .pb_0()
                .child(div().text_lg().child("Settings"))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("Manage your SunSynk account and monitoring preferences."),
                ),
        )
        .child(settings_section(
            theme,
            "SunSynk account",
            div()
                .v_flex()
                .gap_1()
                .child(field("Email address", Input::new(email)))
                .child(field("Password", Input::new(password).mask_toggle()))
                .child(connect_control(
                    theme,
                    email,
                    password,
                    refresh_interval,
                    entity.clone(),
                    connection,
                    fetching,
                )),
        ))
        .child(settings_section(
            theme,
            "Inverters",
            inverter_list(theme, inverters, connection, selected, entity.clone()),
        ))
        .child(settings_section(
            theme,
            "Monitoring",
            div()
                .v_flex()
                .gap_1()
                .child(field(
                    "Refresh interval (seconds)",
                    Input::new(refresh_interval),
                ))
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .v_flex()
                                        .gap_1()
                                        .child(div().text_sm().child("Launch at startup"))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child(
                                                    "Start SunTray in the tray when you sign in.",
                                                ),
                                        ),
                                )
                                .child({
                                    let entity = entity.clone();
                                    Switch::new("launch-at-startup")
                                        .checked(startup_enabled)
                                        .small()
                                        .on_click(move |enabled, _, cx| {
                                            entity.update(cx, |dashboard, cx| {
                                                dashboard.set_startup_enabled(*enabled, cx);
                                            });
                                        })
                                }),
                        )
                        .when_some(startup_error, |element, error| {
                            element.child(div().text_xs().text_color(theme.danger).child(error))
                        }),
                ),
        ))
        .child(settings_section(
            theme,
            "System tray",
            tray_metric_control(theme, tray_metric, entity.clone()),
        ))
        .into_any_element()
}

fn settings_section(theme: &Theme, title: &str, content: impl IntoElement) -> impl IntoElement {
    div()
        .v_flex()
        .gap_2()
        .py_2()
        .border_b_1()
        .border_color(theme.border)
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.muted_foreground)
                .child(title.to_owned()),
        )
        .child(content)
}

pub(crate) struct SettingsView<'a> {
    pub(crate) theme: &'a Theme,
    pub(crate) email: &'a Entity<InputState>,
    pub(crate) password: &'a Entity<InputState>,
    pub(crate) refresh_interval: &'a Entity<InputState>,
    pub(crate) connection: &'a crate::app::ConnectionState,
    pub(crate) inverters: &'a [InverterSummary],
    pub(crate) selected: &'a Option<String>,
    pub(crate) tray_metric: Option<TrayMetric>,
    pub(crate) fetching: bool,
    pub(crate) startup_enabled: bool,
    pub(crate) startup_error: Option<String>,
    pub(crate) entity: Entity<Dashboard>,
}

pub(crate) fn field(label: &str, input: Input) -> impl IntoElement {
    div()
        .v_flex()
        .gap_1()
        .child(div().text_sm().child(label.to_owned()))
        .child(input.small())
}

pub(crate) fn tray_metric_control(
    theme: &Theme,
    selected: Option<TrayMetric>,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    let options = [
        ("None", None),
        ("SoC", Some(TrayMetric::Soc)),
        ("Load", Some(TrayMetric::Load)),
        ("Solar", Some(TrayMetric::Solar)),
    ];
    let mut buttons = div().h_flex().gap_1();
    for (index, (label, metric)) in options.into_iter().enumerate() {
        let target = entity.clone();
        buttons = buttons.child(
            Button::new(("tray-metric", index))
                .label(label)
                .when(selected == metric, |button| button.primary())
                .xsmall()
                .on_click(move |_, _, cx| {
                    target.update(cx, |dashboard, cx| dashboard.set_tray_metric(metric, cx));
                }),
        );
    }
    div()
        .v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Choose what SunTray shows in the system tray."),
        )
        .child(buttons)
}

pub(crate) fn connect_control(
    theme: &Theme,
    email: &Entity<InputState>,
    password: &Entity<InputState>,
    refresh_interval: &Entity<InputState>,
    entity: Entity<Dashboard>,
    connection: &crate::app::ConnectionState,
    fetching: bool,
) -> impl IntoElement {
    let label = match connection {
        crate::app::ConnectionState::Connecting => "Connecting…",
        crate::app::ConnectionState::Connected | crate::app::ConnectionState::Stale => "Reconnect",
        _ => "Connect account",
    };
    let message = match connection {
        crate::app::ConnectionState::Error(error) => Some(error.clone()),
        crate::app::ConnectionState::Connected => None,
        crate::app::ConnectionState::Stale => {
            Some("Connection lost. Cached data is being shown.".into())
        }
        _ => None,
    };
    div().v_flex().gap_1().child(
        div()
            .h_flex()
            .items_center()
            .gap_2()
            .child(
                Button::new("connect")
                    .label(label)
                    .primary()
                    .small()
                    .loading(fetching)
                    .disabled(fetching)
                    .on_click({
                        let email = email.clone();
                        let password = password.clone();
                        let refresh_interval = refresh_interval.clone();
                        let entity = entity.clone();
                        move |_, _, cx| {
                            let email = email.read(cx).value().to_string();
                            let password = password.read(cx).value().to_string();
                            let refresh_seconds = refresh_interval
                                .read(cx)
                                .value()
                                .parse::<u64>()
                                .unwrap_or(60)
                                .clamp(1, 3600);
                            entity.update(cx, |dashboard, cx| {
                                dashboard.reconnect_or_connect(
                                    email,
                                    password,
                                    refresh_seconds,
                                    cx,
                                );
                            });
                        }
                    }),
            )
            .when_some(message, |element, message| {
                element.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(message),
                )
            }),
    )
}

pub(crate) fn inverter_list(
    theme: &Theme,
    inverters: &[InverterSummary],
    connection: &crate::app::ConnectionState,
    selected: &Option<String>,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    let mut list = div().v_flex().gap_0();
    if matches!(connection, crate::app::ConnectionState::Connecting) {
        return list.child(
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("Discovering plants and inverters…"),
        );
    }
    if let crate::app::ConnectionState::Error(_) = connection {
        return list.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Could not load inverters. Reconnect to try again."),
        );
    }
    if inverters.is_empty() {
        return list.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("No inverters available."),
        );
    }
    for (index, inverter) in inverters.iter().enumerate() {
        let is_selected = selected.as_deref() == Some(inverter.serial.as_str());
        let serial = inverter.serial.clone();
        let target = entity.clone();
        let title = if inverter.alias.is_empty() {
            inverter.serial.clone()
        } else {
            inverter.alias.clone()
        };
        list =
            list.child(
                div()
                    .h_flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1()
                    .child(
                        div()
                            .v_flex()
                            .flex_1()
                            .child(div().text_sm().child(title))
                            .child(div().text_xs().text_color(theme.muted_foreground).child(
                                format!(
                                    "{} · {}",
                                    inverter.serial,
                                    if inverter.model.is_empty() {
                                        if inverter.plant_name.is_empty() {
                                            "No plant"
                                        } else {
                                            &inverter.plant_name
                                        }
                                    } else {
                                        &inverter.model
                                    }
                                ),
                            )),
                    )
                    .child(div().text_xs().text_color(theme.muted_foreground).child(
                        if inverter.status == 1 {
                            "Online"
                        } else {
                            "Offline"
                        },
                    ))
                    .child(
                        Button::new(("inverter", index))
                            .label(if is_selected { "Selected" } else { "Use" })
                            .when(is_selected, |button| button.primary())
                            .small()
                            .on_click(move |_, _, cx| {
                                target.update(cx, |dashboard, cx| {
                                    dashboard.select_inverter(serial.clone(), cx)
                                });
                            }),
                    ),
            );
    }
    list
}
