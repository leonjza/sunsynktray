use crate::app::Dashboard;
use gpui_kit::component::calendar::Date;
use gpui_kit::*;

impl Dashboard {
    pub(crate) fn change_history_day(
        &mut self,
        offset: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.controller.update(cx, |controller, cx| {
            controller.change_history_day(offset, cx)
        });
        let date = self.controller.read(cx).history_date;
        self.history_date_picker.update(cx, |picker, cx| {
            picker.set_date(Date::Single(Some(date)), window, cx);
        });
    }

    pub(crate) fn select_history_date(&mut self, date: chrono::NaiveDate, cx: &mut Context<Self>) {
        self.controller.update(cx, |controller, cx| {
            controller.select_history_date(date, cx)
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
