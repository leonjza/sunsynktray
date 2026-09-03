use crate::{app::polling::protocol::Command as PollCommand, app::Dashboard};
use gpui::*;

impl Dashboard {
    pub(crate) fn change_history_day(&mut self, offset: i64, cx: &mut Context<Self>) {
        if self.fetching {
            return;
        }
        let today = chrono::Local::now().date_naive();
        let date = self.history_date + chrono::Duration::days(offset);
        if date > today || date == self.history_date {
            return;
        }
        let Some(sender) = &self.poll_sender else {
            self.activity = "Historical data is unavailable until connected".into();
            cx.notify();
            return;
        };
        self.history_previous_date = Some(self.history_date);
        self.history_date = date;
        self.history_is_manual = date != today;
        self.fetching = true;
        self.activity = "Fetching historical data…".into();
        if sender.try_send(PollCommand::HistoryDate(date)).is_err() {
            self.history_date = self.history_previous_date.take().unwrap_or(today);
            self.fetching = false;
            self.activity = "Polling is unavailable".into();
        }
        cx.notify();
    }

    pub(crate) fn hover_history(&mut self, index: Option<usize>, cx: &mut Context<Self>) {
        if self.hovered_history == index {
            return;
        }
        self.hovered_history = index;
        cx.notify();
    }
}
