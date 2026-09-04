use crate::{app::Dashboard, domain::EnergySnapshot, ui::format::format_power};
use gpui_kit::component::{Icon, IconName, StyledExt, Theme};
use gpui_kit::*;
use std::time::Duration;

pub(crate) fn render(
    theme: &Theme,
    snapshot: &EnergySnapshot,
    connected: bool,
    fetching: bool,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    let battery_direction = (snapshot.battery_watts.abs() > 1.0).then(|| {
        snapshot
            .battery_to
            .or(snapshot.to_battery.map(|to_battery| !to_battery))
            .unwrap_or(false)
    });
    let battery_color = if battery_direction == Some(true) {
        rgb(0xf5b942).into()
    } else {
        rgb(0x34c759).into()
    };

    div()
        .h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .pt_2()
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(flow_node(
                    theme,
                    Icon::new(IconName::Sun),
                    "Solar",
                    format_power(snapshot.pv_watts),
                    snapshot.pv_watts.abs() > 1.,
                    if snapshot.pv_watts.abs() <= 1. {
                        "Idle"
                    } else {
                        "Input"
                    },
                ))
                .child(flow_node(
                    theme,
                    Icon::empty().path("icons/battery.svg"),
                    "Battery",
                    format!("{:.0}%", snapshot.battery_soc),
                    snapshot.battery_watts.abs() > 1.,
                    if snapshot.battery_watts.abs() <= 1. {
                        "Idle".to_owned()
                    } else {
                        format_power(snapshot.battery_watts)
                    },
                )),
        )
        .child(flow_connector(
            theme,
            (snapshot.pv_watts.abs() > 1.0)
                .then(|| snapshot.pv_to.unwrap_or(snapshot.pv_watts > 0.0)),
            battery_direction,
            ["flow-solar", "flow-battery"],
            [0.0, 0.25],
            [rgb(0x34c759).into(), battery_color],
        ))
        .child(inverter_node(theme, connected, fetching, entity))
        .child(flow_connector(
            theme,
            (snapshot.load_watts.abs() > 1.0).then(|| snapshot.to_load.unwrap_or(true)),
            (snapshot.grid_watts.abs() > 1.0)
                .then(|| snapshot.to_grid.unwrap_or(snapshot.grid_watts < 0.0)),
            ["flow-home", "flow-grid"],
            [0.5, 0.75],
            [
                rgb(0x34c759).into(),
                if snapshot.grid_watts > 1.0 {
                    rgb(0xf5b942).into()
                } else {
                    rgb(0x34c759).into()
                },
            ],
        ))
        .child(
            div()
                .v_flex()
                .gap_2()
                .child(flow_node(
                    theme,
                    Icon::empty().path("icons/house.svg"),
                    "Home",
                    format_power(snapshot.load_watts),
                    snapshot.load_watts.abs() > 1.,
                    if snapshot.load_watts.abs() <= 1. {
                        "Idle"
                    } else {
                        "Load"
                    },
                ))
                .child(flow_node(
                    theme,
                    Icon::empty().path("icons/grid.svg"),
                    "Grid",
                    format_power(snapshot.grid_watts.abs()),
                    snapshot.grid_watts.abs() > 1.,
                    if snapshot.grid_watts.abs() <= 1. {
                        "Idle"
                    } else if snapshot.grid_watts < 0. {
                        "Exporting"
                    } else {
                        "Importing"
                    },
                )),
        )
}

fn flow_node(
    theme: &Theme,
    icon: Icon,
    label: &str,
    value: String,
    active: bool,
    detail: impl Into<SharedString>,
) -> impl IntoElement {
    div()
        .v_flex()
        .w(px(132.))
        .h(px(80.))
        .justify_center()
        .gap_0()
        .px_3()
        .py_2()
        .border_1()
        .border_color(theme.border)
        .rounded(theme.radius)
        .child(
            div()
                .h_flex()
                .gap_2()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(icon.size_4().text_color(if active {
                    theme.foreground
                } else {
                    theme.muted_foreground
                }))
                .child(label.to_owned()),
        )
        .child(div().text_lg().font_weight(FontWeight::MEDIUM).child(value))
        .child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(detail.into()),
        )
}

fn inverter_node(
    theme: &Theme,
    connected: bool,
    fetching: bool,
    entity: Entity<Dashboard>,
) -> impl IntoElement {
    let mut node = div()
        .v_flex()
        .items_center()
        .justify_center()
        .w(px(76.))
        .h(px(144.))
        .bg(theme.muted)
        .rounded(theme.radius);
    node.interactivity().on_click(move |_, _, cx| {
        entity.update(cx, |dashboard, cx| dashboard.refresh_now(cx));
    });
    let status_color = if connected {
        rgb(0x34c759).into()
    } else {
        theme.muted_foreground
    };
    let mut status = div()
        .relative()
        .size_3()
        .flex_none()
        .child(div().absolute().inset_0().rounded_full().bg(status_color));
    if fetching {
        status = div()
            .relative()
            .size_3()
            .flex_none()
            .h_flex()
            .items_center()
            .justify_center()
            .child(loading_dots([
                "refresh-inverter-1",
                "refresh-inverter-2",
                "refresh-inverter-3",
            ]));
    }
    node.child(status)
        .child(div().text_xs().mt_2().child("Inverter"))
}

fn loading_dots(ids: [&'static str; 3]) -> impl IntoElement {
    let animation = || {
        Animation::new(Duration::from_millis(650))
            .repeat()
            .with_easing(ease_in_out)
    };
    div()
        .h_flex()
        .items_center()
        .gap_1()
        .size_4()
        .children(ids.into_iter().map(|id| {
            div()
                .size_1()
                .rounded_full()
                .bg(rgb(0x34c759))
                .with_animation(id, animation(), |this, delta| {
                    this.opacity(0.3 + (delta * 0.7))
                })
        }))
}

fn flow_connector(
    theme: &Theme,
    top_forward: Option<bool>,
    bottom_forward: Option<bool>,
    ids: [&'static str; 2],
    phases: [f32; 2],
    colors: [Hsla; 2],
) -> impl IntoElement {
    div()
        .v_flex()
        .flex_1()
        .gap_2()
        .px_3()
        .text_color(theme.muted_foreground)
        .child(flow_arrow(theme, top_forward, ids[0], phases[0], colors[0]))
        .child(flow_arrow(
            theme,
            bottom_forward,
            ids[1],
            phases[1],
            colors[1],
        ))
}

fn flow_arrow(
    theme: &Theme,
    forward: Option<bool>,
    id: &'static str,
    phase: f32,
    color: Hsla,
) -> impl IntoElement {
    let line = flow_line(theme, forward, id, phase, color);
    match forward {
        Some(true) => div()
            .h(px(80.))
            .h_flex()
            .items_center()
            .gap_1()
            .child(line)
            .child(div().text_lg().child("→")),
        Some(false) => div()
            .h(px(80.))
            .h_flex()
            .items_center()
            .gap_1()
            .child(div().text_lg().child("←"))
            .child(line),
        None => div().h(px(80.)).h_flex().items_center().child(line),
    }
}

fn flow_line(
    theme: &Theme,
    forward: Option<bool>,
    id: &'static str,
    phase: f32,
    color: Hsla,
) -> impl IntoElement {
    let line = div().relative().h(px(1.)).flex_1().bg(theme.border);
    let Some(forward) = forward else {
        return line.child(
            div()
                .absolute()
                .left(relative(0.5))
                .top(px(-9.))
                .ml(px(-10.))
                .text_xs()
                .text_color(theme.muted_foreground)
                .child("Idle"),
        );
    };
    let start = if forward { 0. } else { 1. };
    line.child(
        div()
            .absolute()
            .top(px(-2.))
            .left(relative(start))
            .size_1()
            .rounded_full()
            .bg(color)
            .with_animation(
                id,
                Animation::new(Duration::from_millis(2400))
                    .repeat()
                    .with_easing(ease_in_out),
                move |this, delta| {
                    let position = if forward {
                        (delta + phase) % 1.
                    } else {
                        1. - ((delta + phase) % 1.)
                    };
                    this.left(relative(position))
                },
            ),
    )
}
