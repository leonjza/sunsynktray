use crate::{
    app::HistoryPointIndex,
    domain::HistorySeries,
    ui::format::{format_power, history_colors, history_label, history_value, series_color_index},
};
use gpui_component_macros::IntoPlot;
use gpui_kit::base::plot::scale::{Scale, ScaleLinear};
use gpui_kit::base::plot::shape::Line;
use gpui_kit::base::plot::PathCaches;
use gpui_kit::base::plot::{axis_gutter, AxisText, Grid, Plot, PlotAxis, TooltipState};
use gpui_kit::component::{
    plot::tooltip::{CrossLine, Dot, Tooltip},
    ActiveTheme, StyledExt, Theme,
};
use gpui_kit::*;
use std::sync::Arc;

pub(crate) const HEIGHT: f32 = 290.;
const PLOT_LEFT: f32 = 42.;
const PLOT_TOP: f32 = 10.;

#[derive(IntoPlot)]
pub(crate) struct HistoryPlot {
    pub(crate) history: Arc<Vec<HistorySeries>>,
    pub(crate) point_index: Arc<HistoryPointIndex>,
    pub(crate) power_indices: Vec<usize>,
    pub(crate) soc_indices: Vec<usize>,
    pub(crate) times: Arc<Vec<String>>,
    pub(crate) power_bounds: (f64, f64),
    pub(crate) full_day: bool,
}

impl Plot for HistoryPlot {
    fn paint(&mut self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        if self.times.is_empty() {
            return;
        }
        let width = bounds.size.width.as_f32();
        let height = bounds.size.height.as_f32() - axis_gutter(px(10.));
        let plot_bounds = Bounds::new(
            point(bounds.origin.x + px(PLOT_LEFT), bounds.origin.y),
            size(px(width - PLOT_LEFT), bounds.size.height),
        );
        let plot_width = plot_bounds.size.width.as_f32();
        let (min_value, max_value) = self.power_bounds;
        let y = ScaleLinear::new([min_value, max_value], [height, PLOT_TOP]);
        let y_ticks = [min_value, (min_value + max_value) / 2., max_value];
        let domain_end = self.domain_end_minutes();
        let x_ticks = time_ticks(domain_end, self.full_day);
        let x_labels = x_ticks.iter().enumerate().map(|(index, &minute)| {
            let position = x_position(minute, domain_end, plot_width);
            let align = if index == 0 {
                TextAlign::Left
            } else if index == x_ticks.len() - 1 {
                TextAlign::Right
            } else {
                TextAlign::Center
            };
            AxisText::new(
                format_time(minute),
                position + PLOT_LEFT,
                cx.theme().muted_foreground,
            )
            .align(align)
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
        let soc_y = ScaleLinear::new([0., 100.], [height, PLOT_TOP]);
        let caches = PathCaches::for_paint("history-lines", window, cx);
        caches.update(cx, |caches, _| {
            for (slot, &index) in self
                .power_indices
                .iter()
                .chain(&self.soc_indices)
                .enumerate()
            {
                let series = &self.history[index];
                let y_scale = if self.soc_indices.contains(&index) {
                    soc_y.clone()
                } else {
                    y.clone()
                };
                Line::new()
                    .data(series.points.iter())
                    .x(move |point| {
                        time_of_day_minutes(&point.time)
                            .map(|minute| x_position(minute, domain_end, plot_width))
                    })
                    .y(move |point| y_scale.tick(&point.watts))
                    .stroke(colors[series_color_index(&series.label) % colors.len()])
                    .stroke_width(px(1.5))
                    .paint_cached(&plot_bounds, caches.slot(slot), window);
            }
        });
    }

    fn id(&self) -> Option<ElementId> {
        Some("history-plot".into())
    }

    fn tooltip_state(
        &self,
        position: Point<Pixels>,
        bounds: Bounds<Pixels>,
        _cx: &App,
    ) -> Option<TooltipState> {
        let plot_width = bounds.size.width.as_f32() - PLOT_LEFT;
        let plot_height = bounds.size.height.as_f32() - axis_gutter(px(10.));
        let x = position.x.as_f32() - PLOT_LEFT;
        let y = position.y.as_f32();
        if self.times.is_empty()
            || plot_width <= 0.
            || x < 0.
            || x > plot_width
            || y < PLOT_TOP
            || y > plot_height
        {
            return None;
        }
        let domain_end = self.domain_end_minutes();
        let target_minute = (x / plot_width) * domain_end;
        let (index, minute) = self
            .times
            .iter()
            .enumerate()
            .filter_map(|(index, time)| time_of_day_minutes(time).map(|minute| (index, minute)))
            .min_by(|left, right| {
                (left.1 - target_minute)
                    .abs()
                    .total_cmp(&(right.1 - target_minute).abs())
            })?;
        let (min_value, max_value) = self.power_bounds;
        let power_y_scale = ScaleLinear::new([min_value, max_value], [plot_height, PLOT_TOP]);
        let soc_y_scale = ScaleLinear::new([0., 100.], [plot_height, PLOT_TOP]);
        let time = &self.times[index];
        let dots = self
            .history
            .iter()
            .enumerate()
            .filter_map(|(series_index, _series)| {
                let value = *self.point_index.get(&series_index)?.get(time)?;
                let is_soc = self.soc_indices.contains(&series_index);
                let y = if is_soc {
                    soc_y_scale.tick(&value)?
                } else {
                    power_y_scale.tick(&value)?
                };
                Some(point(
                    px(PLOT_LEFT + x_position(minute, domain_end, plot_width)),
                    px(y),
                ))
            })
            .collect::<Vec<_>>();
        Some(TooltipState::new(
            index,
            point(
                px(PLOT_LEFT + x_position(minute, domain_end, plot_width)),
                position.y,
            ),
            dots,
        ))
    }

    fn tooltip(
        &self,
        state: &TooltipState,
        cursor: Point<Pixels>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyElement> {
        let time = self.times.get(state.index)?;
        let plot_height = bounds.size.height.as_f32() - axis_gutter(px(10.));
        let colors = history_colors();
        let dot_colors = self
            .history
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                self.point_index.get(&index)?.get(time)?;
                Some(colors[series_color_index(&series.label) % colors.len()])
            });
        let mut tooltip = Tooltip::new(cursor, bounds.size)
            .gap(px(8.))
            .cross_line(CrossLine::new(state.cross_line).height(plot_height))
            .dots(state.dots.iter().zip(dot_colors).map(|(point, color)| {
                Dot::new(*point)
                    .size(px(7.))
                    .halo(px(10.))
                    .stroke(cx.theme().background)
                    .fill(color)
            }));
        for (index, series) in self.history.iter().enumerate() {
            if let Some(value) = self
                .point_index
                .get(&index)
                .and_then(|points| points.get(time))
            {
                let color = colors[series_color_index(&series.label) % colors.len()];
                tooltip = tooltip.row(
                    color,
                    history_label(&series.label),
                    history_value(&series.label, *value),
                );
            }
        }
        Some(tooltip.title(time.clone()).into_any_element())
    }
}

impl HistoryPlot {
    fn domain_end_minutes(&self) -> f32 {
        if self.full_day {
            24. * 60.
        } else {
            self.times
                .iter()
                .filter_map(|time| time_of_day_minutes(time))
                .fold(0_f32, f32::max)
                .max(1.)
        }
    }
}

fn time_of_day_minutes(value: &str) -> Option<f32> {
    let mut parts = value.trim().split(':');
    let hour = parts.next()?.parse::<u8>().ok()?;
    let minute = parts.next()?.parse::<u8>().ok()?;
    let second = parts
        .next()
        .map(str::parse::<f32>)
        .transpose()
        .ok()?
        .unwrap_or(0.);
    if parts.next().is_some() || hour > 23 || minute > 59 || !(0. ..60.).contains(&second) {
        return None;
    }
    Some(hour as f32 * 60. + minute as f32 + second / 60.)
}

fn x_position(minute: f32, domain_end: f32, width: f32) -> f32 {
    (minute / domain_end).clamp(0., 1.) * width
}

fn time_ticks(domain_end: f32, full_day: bool) -> Vec<f32> {
    if full_day {
        return vec![0., 240., 480., 720., 960., 1200., 1439.];
    }

    let step = if domain_end <= 240. {
        60.
    } else if domain_end <= 480. {
        120.
    } else {
        240.
    };
    let mut ticks = (0..)
        .map(|index| index as f32 * step)
        .take_while(|&minute| minute <= domain_end)
        .collect::<Vec<_>>();
    if ticks
        .last()
        .is_none_or(|last| domain_end - last >= step / 2.)
    {
        ticks.push(domain_end);
    }
    ticks
}

fn format_time(minute: f32) -> String {
    let minute = minute.round().clamp(0., 1439.) as u16;
    format!("{:02}:{:02}", minute / 60, minute % 60)
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
