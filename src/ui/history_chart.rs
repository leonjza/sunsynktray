use crate::{
    app::Dashboard,
    domain::HistorySeries,
    ui::format::{format_power, history_colors, history_label, history_value, series_color_index},
};
use gpui_component_macros::IntoPlot;
use gpui_kit::component::plot::scale::{Scale, ScaleLinear, ScalePoint};
use gpui_kit::component::plot::shape::Line;
use gpui_kit::component::plot::{AxisText, Grid, Plot, PlotAxis, StrokeStyle};
use gpui_kit::component::{ActiveTheme, StyledExt, Theme};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub(crate) const HEIGHT: f32 = 290.;
const PLOT_LEFT: f32 = 42.;
const PLOT_TOP: f32 = 10.;

#[derive(IntoPlot)]
pub(crate) struct HistoryPlot {
    pub(crate) history: Arc<Vec<HistorySeries>>,
    pub(crate) power_indices: Vec<usize>,
    pub(crate) soc_indices: Vec<usize>,
    pub(crate) times: Vec<String>,
    pub(crate) chart_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
}

impl Plot for HistoryPlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        if let Ok(mut chart_bounds) = self.chart_bounds.lock() {
            *chart_bounds = None;
        }
        if self.times.is_empty() {
            return;
        }
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - gpui_kit::component::plot::AXIS_GAP;
        let plot_bounds = Bounds::new(
            point(bounds.origin.x + px(PLOT_LEFT), bounds.origin.y),
            size(px(width - PLOT_LEFT), bounds.size.height),
        );
        if let Ok(mut chart_bounds) = self.chart_bounds.lock() {
            *chart_bounds = Some(plot_bounds);
        }
        let x = ScalePoint::new(
            self.times.clone(),
            vec![0., plot_bounds.size.width.as_f32()],
        );
        let (min_value, max_value) = power_bounds(&self.history, &self.power_indices);
        let y = ScaleLinear::new(vec![min_value, max_value], vec![height, PLOT_TOP]);
        let y_ticks = [min_value, (min_value + max_value) / 2., max_value];
        let tick_margin = (self.times.len() / 5).max(1);
        let x_labels = self.times.iter().enumerate().filter_map(|(index, label)| {
            (index % tick_margin == 0)
                .then(|| {
                    x.tick(label).map(|position| {
                        let align = if index == 0 {
                            TextAlign::Left
                        } else if index == self.times.len() - 1 {
                            TextAlign::Right
                        } else {
                            TextAlign::Center
                        };
                        AxisText::new(
                            label.clone(),
                            position + PLOT_LEFT,
                            cx.theme().muted_foreground,
                        )
                        .align(align)
                    })
                })
                .flatten()
        });
        let y_labels = y_ticks.iter().filter_map(|value| {
            y.tick(value).map(|position| {
                AxisText::new(
                    format_power(*value),
                    position - 8.,
                    cx.theme().muted_foreground,
                )
                .align(TextAlign::Right)
            })
        });
        PlotAxis::new()
            .x(height)
            .x_label(x_labels)
            .x_axis(false)
            .y(PLOT_LEFT - 8.)
            .y_label(y_labels)
            .stroke(cx.theme().border)
            .paint(&bounds, window, cx);
        let grid_lines = y_ticks
            .iter()
            .filter_map(|value| y.tick(value))
            .collect::<Vec<_>>();
        Grid::new()
            .y(grid_lines)
            .stroke(cx.theme().border)
            .dash_array(&[px(4.), px(2.)])
            .paint(&plot_bounds, window);
        let colors = history_colors();
        for &index in &self.power_indices {
            let series = &self.history[index];
            let x_scale = x.clone();
            let y_scale = y.clone();
            Line::new()
                .data(series.points.iter())
                .x(move |point| x_scale.tick(&point.time))
                .y(move |point| y_scale.tick(&point.watts))
                .stroke(colors[series_color_index(&series.label) % colors.len()])
                .stroke_style(StrokeStyle::Natural)
                .stroke_width(px(1.5))
                .paint(&plot_bounds, window);
        }
        let soc_y = ScaleLinear::new(vec![0., 100.], vec![height, PLOT_TOP]);
        for &index in &self.soc_indices {
            let series = &self.history[index];
            let x_scale = x.clone();
            let y_scale = soc_y.clone();
            Line::new()
                .data(series.points.iter())
                .x(move |point| x_scale.tick(&point.time))
                .y(move |point| y_scale.tick(&point.watts))
                .stroke(colors[series_color_index(&series.label) % colors.len()])
                .stroke_style(StrokeStyle::Natural)
                .stroke_width(px(1.5))
                .paint(&plot_bounds, window);
        }
    }
}

pub(crate) fn times(history: &[HistorySeries]) -> Vec<String> {
    let mut times = Vec::new();
    let mut seen = HashSet::new();
    for point in history.iter().flat_map(|series| &series.points) {
        if seen.insert(point.time.clone()) {
            times.push(point.time.clone());
        }
    }
    times.sort_unstable();
    times
}

pub(crate) fn power_bounds(history: &[HistorySeries], indices: &[usize]) -> (f64, f64) {
    indices
        .iter()
        .filter_map(|&index| history.get(index))
        .flat_map(|series| series.points.iter().map(|point| point.watts))
        .chain(Some(0.))
        .fold((0.0_f64, 0.0_f64), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

fn power_y(value: f64, min: f64, max: f64, height: f32) -> f32 {
    if (max - min).abs() < f64::EPSILON {
        height / 2.
    } else {
        height - ((value - min) as f32 / (max - min) as f32) * (height - PLOT_TOP)
    }
}

pub(crate) fn legend(theme: &Theme, history: &[HistorySeries]) -> impl IntoElement {
    let colors = history_colors();
    let mut legend = div().h_flex().gap_3();
    for series in history {
        legend = legend.child(
            div()
                .h_flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .size_2()
                        .rounded(theme.radius)
                        .bg(colors[series_color_index(&series.label) % colors.len()]),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(history_label(&series.label)),
                ),
        );
    }
    legend
}

pub(crate) fn hover_layer(
    theme: &Theme,
    history: &[HistorySeries],
    power_indices: &[usize],
    entity: Entity<Dashboard>,
    times: &[String],
    hovered: Option<usize>,
) -> impl IntoElement {
    let (min_value, max_value) = power_bounds(history, power_indices);
    let chart_height = HEIGHT - gpui_kit::component::plot::AXIS_GAP;
    let mut layer = div()
        .absolute()
        .top_0()
        .left(px(PLOT_LEFT))
        .right_0()
        .bottom_0()
        .h_flex();
    let hover_entity = entity.clone();
    layer.interactivity().on_hover(move |is_hovered, _, cx| {
        if !*is_hovered {
            hover_entity.update(cx, |dashboard, cx| dashboard.hover_history(None, cx));
        }
    });
    for index in 0..times.len() {
        let cell_entity = entity.clone();
        let hover_time = times.get(index).map(String::as_str);
        let mut cell = div().flex_1().h_full().relative();
        if hovered == Some(index) {
            let colors = history_colors();
            let power_dots = power_indices.iter().filter_map(|&index| {
                let series = history.get(index)?;
                let point = series
                    .points
                    .iter()
                    .find(|point| Some(point.time.as_str()) == hover_time)?;
                let y = power_y(point.watts, min_value, max_value, chart_height);
                Some((y, colors[series_color_index(&series.label) % colors.len()]))
            });
            let soc_dots = history
                .iter()
                .filter(|series| series.label.to_ascii_lowercase().contains("soc"))
                .filter_map(|series| {
                    let value = series
                        .points
                        .iter()
                        .find(|point| Some(point.time.as_str()) == hover_time)?
                        .watts
                        .clamp(0., 100.);
                    Some((
                        chart_height - (value as f32 / 100.) * (chart_height - PLOT_TOP),
                        colors[series_color_index(&series.label) % colors.len()],
                    ))
                });
            let mut dots = div().absolute().top_0().left_0().right_0().bottom_0();
            for (y, color) in power_dots.chain(soc_dots) {
                dots = dots.child(
                    div()
                        .absolute()
                        .left(relative(0.5))
                        .top(px(y - 3.5))
                        .ml(px(-3.5))
                        .size(px(7.))
                        .rounded_full()
                        .border_1()
                        .border_color(theme.background)
                        .bg(color),
                );
            }
            let mut values = div().v_flex().gap_1();
            for series in history {
                if let Some(point) = series
                    .points
                    .iter()
                    .find(|point| Some(point.time.as_str()) == hover_time)
                {
                    values = values.child(
                        div()
                            .h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(colors[series_color_index(&series.label) % colors.len()]),
                            )
                            .child(div().text_xs().child(format!(
                                "{}  {}",
                                history_label(&series.label),
                                history_value(&series.label, point.watts)
                            ))),
                    );
                }
            }
            let card = div()
                .absolute()
                .top(px((chart_height / 2. - 72.).max(8.)))
                .w(px(168.))
                .p_2()
                .border_1()
                .border_color(theme.border)
                .rounded_sm()
                .bg(theme.background.opacity(0.94))
                .when(index < times.len() / 2, |element| element.left_2())
                .when(index >= times.len() / 2, |element| element.right_2())
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(hover_time.unwrap_or_default().to_owned()),
                        )
                        .child(values),
                );
            cell = cell
                .when(true, |element| {
                    element.bg(gpui_kit::transparent_white().opacity(0.03))
                })
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(relative(0.5))
                        .ml(px(-0.5))
                        .w(px(1.))
                        .bg(theme.border),
                )
                .child(dots)
                .child(card);
        }
        cell = cell.on_mouse_move(move |_, _, cx| {
            cell_entity.update(cx, |dashboard, cx| dashboard.hover_history(Some(index), cx));
        });
        let exit_entity = entity.clone();
        cell.interactivity().on_hover(move |is_hovered, _, cx| {
            if !*is_hovered {
                exit_entity.update(cx, |dashboard, cx| dashboard.hover_history(None, cx));
            }
        });
        layer = layer.child(cell);
    }
    layer
}
