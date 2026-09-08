use crate::{
    app::{Dashboard, TrayMetric},
    domain::InverterSummary,
};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    progress::Progress,
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
        history_days,
        connection,
        inverters,
        selected,
        tray_metric,
        startup_enabled,
        startup_pending,
        startup_error,
        refresh_interval_error,
        backfill_completed,
        backfill_total,
        backfill_running,
        backfill_detail,
        next_request_in,
        entity,
    } = view;
    div()
        .v_flex()
        .flex_1()
        .overflow_y_scrollbar()
        .p_3()
        .pb_8()
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
            "Connect a SunSynk account",
            div()
                .v_flex()
                .gap_1()
                .p_3()
                .rounded(theme.radius)
                .bg(theme.muted)
                .child(field("Email address", Input::new(email)))
                .child(field("Password", Input::new(password).mask_toggle()))
                .child(connect_control(
                    theme,
                    email,
                    password,
                    refresh_interval,
                    history_days,
                    entity.clone(),
                    connection,
                )),
        ))
        .child(settings_section(
            theme,
            "Inverters",
            inverter_list(theme, inverters, connection, selected, entity.clone()),
        ))
        .child(settings_section(
            theme,
            "Historical data",
            div()
                .v_flex()
                .gap_3()
                .child(field(
                    "History range (days)",
                    Input::new(history_days).small().w(px(120.)),
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child("SunTray gradually fills this range in the background."),
                )
                .child(backfill_panel(
                    theme,
                    backfill_completed,
                    backfill_total,
                    backfill_running,
                    backfill_detail,
                    next_request_in,
                    entity.clone(),
                )),
        ))
        .child(settings_section(
            theme,
            "Monitoring",
            div().v_flex().gap_1().child(
                div()
                    .v_flex()
                    .gap_1()
                    .child(field(
                        "Refresh interval (seconds)",
                        Input::new(refresh_interval),
                    ))
                    .when_some(refresh_interval_error, |element, error| {
                        element.child(div().text_xs().text_color(theme.danger).child(error))
                    }),
            ),
        ))
        .child(settings_section(
            theme,
            "System",
            div()
                .v_flex()
                .gap_3()
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
                                        .child("Start SunTray in the tray when you sign in."),
                                ),
                        )
                        .child({
                            let entity = entity.clone();
                            Switch::new("launch-at-startup")
                                .checked(startup_enabled)
                                .disabled(startup_pending)
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
                })
                .child(tray_metric_control(theme, tray_metric, entity.clone())),
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

#[allow(clippy::too_many_arguments)]
fn backfill_panel(
    theme: &Theme,
    completed: u64,
    total: u64,
    running: bool,
    detail: String,
    next_request_in: Option<u64>,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    let percent = if total == 0 {
        100.
    } else {
        (completed as f32 / total as f32 * 100.).clamp(0., 100.)
    };
    let status = if total == 0 || completed >= total {
        "Up to date"
    } else if running {
        "Running"
    } else {
        "Paused"
    };
    let status_color = if running || status == "Up to date" {
        rgb(0x22c55e)
    } else {
        rgb(0xa1a1aa)
    };
    let status_background = if running {
        rgb(0x123b25)
    } else {
        rgb(0x27272a)
    };
    let detail = if total == 0 && detail.is_empty() {
        "No historical days need backfilling".to_owned()
    } else if detail.is_empty() {
        "Waiting to start".to_owned()
    } else {
        detail
    };
    let request_in_progress = detail.starts_with("Processing ");
    let detail = (detail != status && detail != "Paused").then_some(detail);
    let next_request = if request_in_progress {
        "Request in progress".to_owned()
    } else {
        next_request_in
            .map(|seconds| {
                if seconds == 0 {
                    "Starting request…".to_owned()
                } else {
                    format!("Next request in {seconds}s")
                }
            })
            .unwrap_or_else(|| "Waiting for next request".to_owned())
    };
    let toggle = entity.clone();

    div()
        .v_flex()
        .gap_3()
        .p_3()
        .rounded(theme.radius)
        .bg(theme.muted)
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .child(div().text_sm().child("Backfill status"))
                .child(
                    Button::new("backfill-toggle")
                        .label(if running { "Pause" } else { "Start" })
                        .small()
                        .disabled(total == 0 || completed >= total)
                        .on_click(move |_, _, cx| {
                            toggle.update(cx, |dashboard, cx| dashboard.toggle_backfill(cx))
                        }),
                ),
        )
        .child(
            Progress::new("history-backfill-progress")
                .value(percent)
                .color(status_color)
                .small()
                .accessibility_label("Historical backfill progress"),
        )
        .child(
            div()
                .h_flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_full()
                                        .bg(status_background)
                                        .text_xs()
                                        .text_color(status_color)
                                        .child(status),
                                )
                                .child(div().text_xs().text_color(theme.muted_foreground).child(
                                    if total == 0 {
                                        "No inverter-days pending".to_owned()
                                    } else {
                                        format!("{completed} of {total} days · {percent:.0}%")
                                    },
                                )),
                        )
                        .when_some(detail, |element, detail| {
                            element.child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(detail)
                                    .when(running, |element| {
                                        element.child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.muted_foreground)
                                                .child(next_request),
                                        )
                                    }),
                            )
                        }),
                ),
        )
}

pub(crate) struct SettingsView<'a> {
    pub(crate) theme: &'a Theme,
    pub(crate) email: &'a Entity<InputState>,
    pub(crate) password: &'a Entity<InputState>,
    pub(crate) refresh_interval: &'a Entity<InputState>,
    pub(crate) history_days: &'a Entity<InputState>,
    pub(crate) connection: &'a crate::app::ConnectionState,
    pub(crate) inverters: &'a [InverterSummary],
    pub(crate) selected: &'a Option<String>,
    pub(crate) tray_metric: Option<TrayMetric>,
    pub(crate) startup_enabled: bool,
    pub(crate) startup_pending: bool,
    pub(crate) startup_error: Option<String>,
    pub(crate) refresh_interval_error: Option<String>,
    pub(crate) backfill_completed: u64,
    pub(crate) backfill_total: u64,
    pub(crate) backfill_running: bool,
    pub(crate) backfill_detail: String,
    pub(crate) next_request_in: Option<u64>,
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn connect_control(
    theme: &Theme,
    email: &Entity<InputState>,
    password: &Entity<InputState>,
    refresh_interval: &Entity<InputState>,
    history_days: &Entity<InputState>,
    entity: Entity<Dashboard>,
    connection: &crate::app::ConnectionState,
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
            Some("Live connection unavailable. Cached data is shown while retrying.".into())
        }
        _ => None,
    };
    let connecting = matches!(connection, crate::app::ConnectionState::Connecting);
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
                    .loading(connecting)
                    .disabled(connecting)
                    .on_click({
                        let email = email.clone();
                        let password = password.clone();
                        let refresh_interval = refresh_interval.clone();
                        let history_days = history_days.clone();
                        let entity = entity.clone();
                        move |_, _, cx| {
                            let email = email.read(cx).value().to_string();
                            let password = password.read(cx).value().to_string();
                            let value = refresh_interval.read(cx).value().to_string();
                            let history_value = history_days.read(cx).value().to_string();
                            let Ok(refresh_seconds) = value.parse::<u64>() else {
                                entity.update(cx, |dashboard, cx| {
                                    dashboard.set_refresh_interval_error(
                                        Some("Enter a whole number of seconds.".into()),
                                        cx,
                                    );
                                });
                                return;
                            };
                            if !(1..=3600).contains(&refresh_seconds) {
                                entity.update(cx, |dashboard, cx| {
                                    dashboard.set_refresh_interval_error(
                                        Some("Use a value between 1 and 3600 seconds.".into()),
                                        cx,
                                    );
                                });
                                return;
                            }
                            let Ok(history_days) = history_value.parse::<u64>() else {
                                return;
                            };
                            if !(1..=3650).contains(&history_days) {
                                return;
                            }
                            entity.update(cx, |dashboard, cx| {
                                dashboard.reconnect(
                                    email,
                                    password,
                                    refresh_seconds,
                                    history_days,
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
