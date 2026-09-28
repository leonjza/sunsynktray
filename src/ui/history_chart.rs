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
    pub(crate) time_indices: Arc<std::collections::HashMap<String, usize>>,
    pub(crate) power_bounds: (f64, f64),
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
        let tick_margin = (self.times.len() / 5).max(1);
        let x_labels = self.times.iter().enumerate().filter_map(|(index, label)| {
            (index % tick_margin == 0)
                .then(|| {
                    Some(x_position(index, self.times.len(), plot_width)).map(|position| {
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
                let time_indices = self.time_indices.clone();
                let times_len = self.times.len();
                let y_scale = if self.soc_indices.contains(&index) {
                    soc_y.clone()
                } else {
                    y.clone()
                };
                Line::new()
                    .data(series.points.iter())
                    .x(move |point| {
                        time_indices
                            .get(&point.time)
                            .map(|&index| x_position(index, times_len, plot_width))
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
        let index =
            ((x / plot_width) * (self.times.len().saturating_sub(1) as f32)).round() as usize;
        let index = index.min(self.times.len() - 1);
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
                    px(PLOT_LEFT + x_position(index, self.times.len(), plot_width)),
                    px(y),
                ))
            })
            .collect::<Vec<_>>();
        Some(TooltipState::new(
            index,
            point(
                px(PLOT_LEFT + x_position(index, self.times.len(), plot_width)),
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

fn x_position(index: usize, count: usize, width: f32) -> f32 {
    if count <= 1 {
        0.
    } else {
        index as f32 / (count - 1) as f32 * width
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
