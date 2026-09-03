pub(super) mod protocol;
pub(super) mod worker;

use crate::{
    app::polling::protocol::{Command as PollCommand, PollResult as ConnectResult},
    domain::{EnergySnapshot, HistorySeries},
    storage::credentials,
};
use gpui::*;

use super::{ConnectionState, Dashboard};
use crate::app::Screen;

pub(super) fn is_fetch_command(command: &PollCommand) -> bool {
    matches!(command, PollCommand::Refresh | PollCommand::Select(_, _))
}

pub(super) fn should_queue_command(command: &PollCommand, fetching: bool) -> bool {
    !fetching || !is_fetch_command(command)
}

impl Dashboard {
    pub(crate) fn apply_snapshot(
        &mut self,
        snapshot: EnergySnapshot,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
        cx: &mut Context<Self>,
    ) {
        if !self.history_is_manual {
            self.history_date = chrono::Local::now().date_naive();
        }
        self.state.set_snapshot(snapshot);
        self.has_cached_data = true;
        let token_changed = refresh_token != self.refresh_token;
        self.refresh_token = refresh_token.clone();
        if token_changed {
            if let (Some((email, _)), Some(token)) = (&self.credentials, refresh_token.as_deref()) {
                credentials::save_refresh_token_async(email.clone(), token.to_owned());
            }
        }
        if let Some(history) = history {
            self.state.set_history(history);
        }
        self.connection = ConnectionState::Connected;
        self.fetching = false;
        self.update_tray(cx);
        self.next_refresh_in = Some(self.refresh_seconds);
        self.activity = format!(
            "Waiting for next refresh · next refresh in {}s",
            self.refresh_seconds
        );
    }

    pub(crate) fn apply_history(&mut self, history: Vec<HistorySeries>) {
        self.state.set_history(history);
        self.history_previous_date = None;
        self.history_is_manual = self.history_date != chrono::Local::now().date_naive();
        self.fetching = false;
        self.next_refresh_in = Some(self.refresh_seconds);
        self.activity = format!(
            "Waiting for next refresh · next refresh in {}s",
            self.refresh_seconds
        );
    }

    pub(crate) fn apply_stopped(&mut self, error: String, cx: &mut Context<Self>) {
        self.polling = false;
        self.poll_sender = None;
        self.fetching = false;
        self.connection = if self.has_cached_data {
            ConnectionState::Stale
        } else {
            ConnectionState::Error(error.clone())
        };
        self.activity = format!("Polling stopped: {error}");
        self.update_tray(cx);
    }

    pub(crate) fn start_polling(&mut self, cx: &mut Context<Self>) {
        if self.polling {
            return;
        }
        let entity = cx.weak_entity();
        self.poll_generation = self.poll_generation.wrapping_add(1);
        let poll_generation = self.poll_generation;
        let interval = self.refresh_seconds.max(1);
        let details = (
            self.state.settings.api_base_url.clone(),
            self.credentials.clone(),
            self.selected_serial.clone(),
            self.selected_serial.as_ref().and_then(|serial| {
                self.inverters
                    .iter()
                    .find(|inverter| &inverter.serial == serial)
                    .and_then(|inverter| inverter.plant_id)
            }),
            self.refresh_token.clone(),
        );
        let Some((email, password)) = details.1 else {
            return;
        };
        let Some(serial) = details.2 else {
            return;
        };
        self.polling = true;
        let (command_sender, mut receiver) = worker::spawn(protocol::PollConfig {
            base_url: details.0,
            email,
            password,
            serial,
            plant_id: details.3,
            refresh_token: details.4,
            interval_seconds: interval,
        });
        self.poll_sender = Some(command_sender);
        cx.spawn(async move |_, cx| {
            while let Some(result) = receiver.recv().await {
                if entity
                    .update(cx, |dashboard, cx| {
                        if dashboard.poll_generation != poll_generation {
                            return;
                        }
                        match result {
                            ConnectResult::PollStarted => {
                                dashboard.fetching = true;
                                dashboard.activity = "Fetching new data…".into();
                            }
                            ConnectResult::Progress { message } => {
                                dashboard.fetching = true;
                                dashboard.activity = message;
                            }
                            ConnectResult::History(history) => {
                                dashboard.apply_history(history);
                            }
                            ConnectResult::Snapshot {
                                snapshot,
                                refresh_token,
                                history,
                            } => {
                                dashboard.apply_snapshot(snapshot, refresh_token, history, cx);
                            }
                            ConnectResult::Failure {
                                error, retry_in, ..
                            } => {
                                dashboard.connection = if dashboard.has_cached_data {
                                    ConnectionState::Stale
                                } else {
                                    ConnectionState::Error(error)
                                };
                                dashboard.next_refresh_in = retry_in;
                                dashboard.activity = retry_in
                                    .map(|seconds| format!("Refresh failed · retry in {seconds}s"))
                                    .unwrap_or_else(|| "Refresh failed".into());
                                dashboard.fetching = false;
                                dashboard.update_tray(cx);
                            }
                            ConnectResult::HistoryFailure { date, error } => {
                                if dashboard.history_date == date {
                                    if let Some(previous) = dashboard.history_previous_date.take() {
                                        dashboard.history_date = previous;
                                        dashboard.history_is_manual = dashboard.history_date
                                            != chrono::Local::now().date_naive();
                                    }
                                }
                                dashboard.fetching = false;
                                dashboard.activity = format!("History unavailable: {error}");
                            }
                            ConnectResult::Stopped { error } => {
                                dashboard.apply_stopped(error, cx);
                            }
                            ConnectResult::Connected { .. } => {}
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(crate) fn stop_polling(&mut self) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        if let Some(sender) = self.poll_sender.take() {
            let _ = sender.try_send(PollCommand::Stop);
        }
        self.polling = false;
    }

    pub(crate) fn select_inverter(&mut self, serial: String, cx: &mut Context<Self>) {
        if self.fetching {
            return;
        }
        if self.selected_serial.as_deref() == Some(serial.as_str()) {
            self.screen = Screen::Dashboard;
            cx.notify();
            return;
        }
        self.selected_serial = Some(serial.clone());
        self.screen = Screen::Dashboard;
        if let Some((email, _)) = &self.credentials {
            credentials::save_selection_async(email.clone(), serial.clone());
        }
        self.activity = "Fetching new data…".into();
        let plant_id = self
            .inverters
            .iter()
            .find(|inverter| inverter.serial == serial)
            .and_then(|inverter| inverter.plant_id);
        self.send_poll_command(PollCommand::Select(serial, plant_id), cx);
        cx.notify();
    }

    pub(crate) fn refresh_now(&mut self, cx: &mut Context<Self>) {
        self.send_poll_command(PollCommand::Refresh, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::{is_fetch_command, should_queue_command};
    use crate::app::polling::protocol::Command;

    #[test]
    fn refresh_and_selection_are_fetch_commands() {
        assert!(is_fetch_command(&Command::Refresh));
        assert!(is_fetch_command(&Command::Select("serial".into(), Some(1))));
        assert!(!is_fetch_command(&Command::Stop));
        assert!(!is_fetch_command(&Command::HistoryDate(
            chrono::NaiveDate::from_ymd_opt(2026, 9, 3).unwrap()
        )));
    }

    #[test]
    fn fetch_commands_are_coalesced_while_fetching() {
        assert!(!should_queue_command(&Command::Refresh, true));
        assert!(!should_queue_command(
            &Command::Select("serial".into(), Some(1)),
            true
        ));
        assert!(should_queue_command(&Command::Refresh, false));
        assert!(should_queue_command(
            &Command::HistoryDate(chrono::NaiveDate::from_ymd_opt(2026, 9, 3).unwrap()),
            true
        ));
        assert!(should_queue_command(&Command::Stop, true));
    }
}
