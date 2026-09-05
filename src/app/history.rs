use crate::app::Dashboard;
use gpui_kit::*;

impl Dashboard {
    pub(crate) fn change_history_day(&mut self, offset: i64, cx: &mut Context<Self>) {
        self.controller.update(cx, |controller, cx| {
            controller.change_history_day(offset, cx)
        });
    }

    pub(crate) fn hover_history(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        if self.hovered_history == index {
            return;
        }
        self.hovered_history = index;
        cx.notify();
    }
}
