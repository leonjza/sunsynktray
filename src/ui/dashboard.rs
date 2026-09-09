use crate::{
    app::MonitorState,
    app::{ConnectionState, Dashboard, HistorySource},
    domain::InverterSummary,
    ui::format::format_energy,
    ui::{
        history_chart as history_chart_module, power_flow as power_flow_view,
        power_state as power_state_view,
    },
};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    date_picker::{DatePicker, DatePickerState},
    spinner::Spinner,
    FocusableExt, IconName, Sizable, StyledExt, Theme,
};
use gpui_kit::*;
use std::sync::{Arc, Mutex};

#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    theme: &Theme,
    state: &MonitorState,
    connection: &ConnectionState,
    fetching: bool,
    history_date_picker: Entity<DatePickerState>,
    history_source: HistorySource,
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
    let data = state.data_snapshot();
    let snapshot = &data.snapshot;
    let live = matches!(connection, ConnectionState::Connected);
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
                .px_1()
                .pb_3()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div().text_sm().font_weight(FontWeight::MEDIUM).child(
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
                        .icon(IconName::Redo2)
                        .accessibility_label("Refresh dashboard")
                        .tooltip("Refresh dashboard")
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
                    snapshot,
                    live,
                    fetching,
                    entity.clone(),
                )),
        )
        .child(power_state_view::render(theme, snapshot, live))
        .child(history_chart(
            theme,
            data.history,
            history_date_picker,
            history_source,
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

#[allow(clippy::too_many_arguments)]
fn history_chart(
    theme: &Theme,
    history: crate::app::HistorySnapshot,
    history_date_picker: Entity<DatePickerState>,
    source: HistorySource,
    hovered: Option<usize>,
    chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    entity: Entity<Dashboard>,
) -> AnyElement {
    let previous = entity.clone();
    let next = entity.clone();
    let chart_entity = entity.clone();
    let mut chart = div()
        .id("history-chart")
        .relative()
        .w_full()
        .h(px(history_chart_module::HEIGHT))
        .child(history_chart_module::HistoryPlot {
            history: history.series.clone(),
            power_indices: history.power_indices.as_ref().clone(),
            soc_indices: history.soc_indices.as_ref().clone(),
            times: history.times.clone(),
            time_indices: history.time_indices.clone(),
            chart_bounds,
            power_bounds: history.power_bounds,
        })
        .child(history_chart_module::hover_layer(
            theme,
            &history.series,
            &history.power_indices,
            &history.index,
            history.power_bounds,
            entity.clone(),
            &history.times,
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
                        .accessibility_label("Previous day")
                        .tooltip("Previous day")
                        .icon(IconName::ChevronLeft)
                        .small()
                        .w(px(28.))
                        .ghost()
                        .on_click(move |_, window, cx| {
                            previous.update(cx, |dashboard, cx| {
                                dashboard.change_history_day(-1, window, cx)
                            });
                        }),
                )
                .child(
                    DatePicker::new(&history_date_picker)
                        .small()
                        .w(px(126.))
                        .focus_ring(false),
                )
                .child(
                    Button::new("next-day")
                        .accessibility_label("Next day")
                        .tooltip("Next day")
                        .icon(IconName::ChevronRight)
                        .small()
                        .w(px(28.))
                        .ghost()
                        .on_click(move |_, window, cx| {
                            next.update(cx, |dashboard, cx| {
                                dashboard.change_history_day(1, window, cx)
                            });
                        }),
                ),
        )
        .child(chart)
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .justify_end()
                .child(history_source_badge(theme, source))
                .child(div().h(px(14.)).w(px(1.)).bg(theme.border))
                .child(history_chart_module::legend(theme, &history.series)),
        )
        .into_any_element()
}

fn history_source_badge(theme: &Theme, source: HistorySource) -> impl IntoElement {
    let (label, color, background) = match source {
        HistorySource::Cached => ("Cached", theme.muted_foreground, theme.muted),
        HistorySource::Network => ("Network", rgb(0x22c55e).into(), rgb(0x123b25).into()),
    };
    div()
        .px_1()
        .py_0()
        .rounded_full()
        .bg(background)
        .text_xs()
        .text_color(color)
        .child(label)
}
