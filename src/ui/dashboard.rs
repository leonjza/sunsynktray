use crate::{
    app::MonitorState,
    app::{ConnectionState, Dashboard},
    domain::{HistorySeries, InverterSummary},
    ui::format::format_energy,
    ui::{history_chart as history_chart_module, power_flow as power_flow_view},
};
use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    spinner::Spinner,
    IconName, Sizable, StyledExt, Theme,
};
use std::sync::{Arc, Mutex};

#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    theme: &Theme,
    state: &MonitorState,
    connection: &ConnectionState,
    fetching: bool,
    history_date: chrono::NaiveDate,
    hovered_history: Option<usize>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    selected_inverter: Option<&InverterSummary>,
    entity: Entity<Dashboard>,
) -> AnyElement {
    if matches!(connection, ConnectionState::Connecting) {
        return dashboard_placeholder(theme, entity, true, None);
    }
    if matches!(connection, ConnectionState::Unconfigured) {
        return dashboard_placeholder(theme, entity, false, None);
    }
    if let ConnectionState::Error(error) = connection {
        return dashboard_placeholder(theme, entity, false, Some(error));
    }
    let snapshot = state.snapshot();
    let live = matches!(
        connection,
        ConnectionState::Connected | ConnectionState::Stale
    );
    let refresh_entity = entity.clone();
    let identity = selected_inverter.map(|inverter| {
        let name = if !inverter.alias.is_empty() && inverter.alias != inverter.serial {
            inverter.alias.clone()
        } else if !inverter.plant_name.is_empty() {
            inverter.plant_name.clone()
        } else {
            "Inverter".into()
        };
        (name, inverter.serial.clone())
    });
    div()
        .v_flex()
        .flex_1()
        .p_4()
        .gap_4()
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .p_3()
                .bg(theme.muted)
                .rounded(theme.radius)
                .child(div().size_2().rounded_full().bg(if live {
                    rgb(0x34c759).into()
                } else {
                    theme.muted_foreground
                }))
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div().text_sm().child(
                                identity
                                    .as_ref()
                                    .map(|(name, _)| name.clone())
                                    .unwrap_or_else(|| "Inverter".into()),
                            ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("·"),
                        )
                        .child(
                            div().text_xs().text_color(theme.muted_foreground).child(
                                identity
                                    .as_ref()
                                    .map(|(_, serial)| serial.clone())
                                    .unwrap_or_else(|| snapshot.inverter_sn.clone()),
                            ),
                        ),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Solar yield"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .child(format_energy(snapshot.solar_yield_kwh)),
                        ),
                )
                .child(
                    Button::new("refresh")
                        .label("Refresh")
                        .icon(IconName::Redo2)
                        .loading(fetching)
                        .loading_icon(IconName::Redo2)
                        .ghost()
                        .xsmall()
                        .on_click(move |_, _, cx| {
                            refresh_entity.update(cx, |dashboard, cx| dashboard.refresh_now(cx));
                        }),
                ),
        )
        .child(
            div()
                .v_flex()
                .gap_0()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("Power flow"),
                )
                .child(power_flow_view::render(
                    theme,
                    &snapshot,
                    live,
                    fetching,
                    entity.clone(),
                )),
        )
        .child(history_chart(
            theme,
            &state.history(),
            history_date,
            hovered_history,
            chart_bounds,
            entity,
        ))
        .into_any_element()
}

fn dashboard_placeholder(
    theme: &Theme,
    entity: Entity<Dashboard>,
    loading: bool,
    error: Option<&String>,
) -> AnyElement {
    let settings_entity = entity;
    let content = if loading {
        div()
            .h_flex()
            .items_center()
            .gap_2()
            .child(Spinner::new().small())
            .child(div().text_sm().child("Connecting to SunSynk…"))
    } else if let Some(error) = error {
        div()
            .v_flex()
            .items_center()
            .gap_3()
            .child(div().text_sm().child("Could not load your dashboard"))
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(error.clone()),
            )
            .child(
                Button::new("open-settings-error")
                    .label("Open settings")
                    .primary()
                    .small()
                    .on_click(move |_, _, cx| {
                        settings_entity.update(cx, |dashboard, cx| {
                            dashboard.open_settings(cx);
                        });
                    }),
            )
    } else {
        div()
            .v_flex()
            .items_center()
            .gap_3()
            .child(div().text_sm().child("Connect your SunSynk account"))
            .child(
                Button::new("open-settings")
                    .label("Open settings")
                    .primary()
                    .small()
                    .on_click(move |_, _, cx| {
                        settings_entity.update(cx, |dashboard, cx| {
                            dashboard.open_settings(cx);
                        });
                    }),
            )
    };
    div()
        .v_flex()
        .flex_1()
        .items_center()
        .justify_center()
        .child(content)
        .into_any_element()
}

fn history_chart(
    theme: &Theme,
    history: &[HistorySeries],
    date: chrono::NaiveDate,
    hovered: Option<usize>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    entity: Entity<Dashboard>,
) -> AnyElement {
    let power = history
        .iter()
        .filter(|series| !series.label.to_ascii_lowercase().contains("soc"))
        .cloned()
        .collect::<Vec<_>>();
    let soc = history
        .iter()
        .filter(|series| series.label.to_ascii_lowercase().contains("soc"))
        .cloned()
        .collect::<Vec<_>>();
    let times = history_chart_module::times(history);
    let mut plotted = power.clone();
    plotted.extend(soc.iter().cloned());
    let previous = entity.clone();
    let next = entity.clone();
    let chart_entity = entity.clone();
    let mut chart = div()
        .id("history-chart")
        .relative()
        .w_full()
        .h(px(history_chart_module::HEIGHT))
        .child(history_chart_module::HistoryPlot {
            series: power.clone(),
            soc_series: soc.clone(),
            times: times.clone(),
            chart_bounds,
        })
        .child(history_chart_module::hover_layer(
            theme,
            history,
            &power,
            entity.clone(),
            &times,
            hovered,
        ));
    chart.interactivity().on_hover(move |is_hovered, _, cx| {
        if !*is_hovered {
            chart_entity.update(cx, |dashboard, cx| dashboard.hover_history(None, cx));
        }
    });
    div()
        .v_flex()
        .gap_2()
        .child(
            div()
                .h_flex()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child("History"),
                )
                .child(div().flex_1())
                .child(
                    Button::new("previous-day")
                        .label("‹")
                        .ghost()
                        .xsmall()
                        .on_click(move |_, _, cx| {
                            previous
                                .update(cx, |dashboard, cx| dashboard.change_history_day(-1, cx));
                        }),
                )
                .child(div().text_xs().child(date.to_string()))
                .child(
                    Button::new("next-day")
                        .label("›")
                        .ghost()
                        .xsmall()
                        .on_click(move |_, _, cx| {
                            next.update(cx, |dashboard, cx| dashboard.change_history_day(1, cx));
                        }),
                ),
        )
        .child(chart)
        .child(
            div()
                .h_flex()
                .justify_end()
                .child(history_chart_module::legend(theme, &plotted)),
        )
        .into_any_element()
}
