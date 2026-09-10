pub(super) mod protocol;
pub(super) mod worker;

use crate::{
    app::polling::protocol::{Command as PollCommand, PollResult as ConnectResult},
    domain::{EnergySnapshot, HistorySeries},
    storage::credentials,
};
use gpui_kit::*;

use super::{ConnectionState, HistorySource, MonitorController};

pub(super) fn is_fetch_command(command: &PollCommand) -> bool {
    matches!(command, PollCommand::Refresh | PollCommand::Select(_, _))
}

pub(super) fn should_queue_command(command: &PollCommand, fetching: bool) -> bool {
    !fetching || !is_fetch_command(command)
}

impl MonitorController {
    pub(crate) fn toggle_backfill(&mut self, cx: &mut Context<Self>) {
        let command = if self.backfill_running {
            PollCommand::PauseBackfill
        } else {
            PollCommand::ResumeBackfill
        };
        let _ = self.send_poll_command(command, cx);
    }
}

impl MonitorController {
    pub(crate) fn apply_live_data(
        &mut self,
        snapshot: EnergySnapshot,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
        cx: &mut Context<Self>,
    ) {
        let was_connected = matches!(self.connection, ConnectionState::Connected);
        if !self.history_is_manual {
            self.history_date = chrono::Local::now().date_naive();
        }
        let snapshot_serial = snapshot.inverter_sn.clone();
        if let Some(history) = &history {
            self.cache_history(
                snapshot_serial.clone(),
                chrono::Local::now().date_naive(),
                history.clone(),
                HistorySource::Network,
            );
        }
        self.state.set_snapshot(snapshot);
        self.has_cached_data = true;
        if let Some((email, _)) = &self.credentials {
            if let Some(history) = &history {
                self.state.database.save_history(
                    email.clone(),
                    snapshot_serial,
                    chrono::Local::now().date_naive().to_string(),
                    history.clone(),
                );
            }
            self.state
                .database
                .save_snapshot(email.clone(), self.state.snapshot());
        }
        let previous_refresh_token = self.refresh_token.clone();
        let token_changed = refresh_token != self.refresh_token;
        self.refresh_token = refresh_token.clone();
        if token_changed {
            if let (Some((email, _)), Some(token)) = (&self.credentials, refresh_token.as_deref()) {
                credentials::save_refresh_token_async(
                    email.clone(),
                    previous_refresh_token,
                    token.to_owned(),
                );
            }
        }
        if !self.history_is_manual {
            if let Some(history) = history {
                self.history_source = HistorySource::Network;
                self.state.set_history(history);
            }
        }
        if !was_connected {
            self.record_connection_event("Live connection restored");
        }
        self.connection = ConnectionState::Connected;
        self.update_tray(cx);
    }

    pub(crate) fn apply_snapshot(
        &mut self,
        snapshot: EnergySnapshot,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
        cx: &mut Context<Self>,
    ) {
        self.apply_live_data(snapshot, refresh_token, history, cx);
        self.fetching = false;
        self.refresh_generation = self.refresh_generation.wrapping_add(1);
        self.next_refresh_in = Some(self.refresh_seconds);
        self.activity = format!(
            "Waiting for next refresh · next refresh in {}s",
            self.refresh_seconds
        );
    }

    pub(crate) fn apply_history(&mut self, history: Vec<HistorySeries>, source: HistorySource) {
        if let Some(serial) = self.selected_serial.clone() {
            self.cache_history(serial, self.history_date, history.clone(), source);
        }
        self.state.set_history(history);
        self.history_source = source;
        self.history_previous_date = None;
        self.fetching = false;
        self.refresh_generation = self.refresh_generation.wrapping_add(1);
        self.next_refresh_in = Some(self.refresh_seconds);
        self.activity = format!(
            "Waiting for next refresh · next refresh in {}s",
            self.refresh_seconds
        );
    }

    pub(crate) fn apply_stopped(&mut self, error: String, cx: &mut Context<Self>) {
        self.record_connection_event(format!("Polling stopped: {error}"));
        self.polling = false;
        self.poll_sender = None;
        self.poll_cancel = None;
        self.poll_task = None;
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
            self.auth_state.clone(),
        );
        let Some((email, password)) = details.1 else {
            return;
        };
        let Some(serial) = details.2 else {
            return;
        };
        let Some(auth) = details.4 else {
            return;
        };
        self.polling = true;
        let (command_sender, cancel_sender, mut receiver, task) =
            worker::spawn(protocol::PollConfig {
                generation: poll_generation,
                base_url: details.0,
                email,
                password,
                auth,
                serial,
                plant_id: details.3,
                interval_seconds: interval,
                history_days: self.history_days,
                database: self.state.database.clone(),
                connection_log: self.connection_log.clone(),
                connection_log_revision: self.connection_log_revision.clone(),
                connection_log_signal: self.connection_log_signal.clone(),
            });
        self.poll_sender = Some(command_sender);
        self.poll_cancel = Some(cancel_sender);
        self.poll_task = task;
        cx.spawn(async move |_, cx| {
            while let Some(result) = receiver.recv().await {
                if entity
                    .update(cx, |controller, cx| {
                        if controller.poll_generation != poll_generation {
                            return;
                        }
                        match result {
                            ConnectResult::PollStarted => {
                                controller.fetching = true;
                                controller.next_refresh_in = Some(controller.refresh_seconds);
                                controller.activity = "Fetching new data…".into();
                            }
                            ConnectResult::Progress {
                                generation,
                                message,
                            } => {
                                if controller.poll_generation != generation {
                                    return;
                                }
                                controller.fetching = true;
                                controller.activity = message;
                            }
                            ConnectResult::History { history, source } => {
                                controller.apply_history(history, source);
                            }
                            ConnectResult::Snapshot {
                                snapshot,
                                refresh_token,
                                history,
                            } => {
                                controller.apply_snapshot(snapshot, refresh_token, history, cx);
                            }
                            ConnectResult::BackfillProgress {
                                completed,
                                total,
                                running,
                                detail,
                                next_request_in,
                            } => {
                                controller.backfill_completed = completed;
                                controller.backfill_total = total;
                                controller.backfill_running = running;
                                controller.backfill_detail = detail;
                                controller.backfill_next_request_in = next_request_in;
                            }
                            ConnectResult::Failure {
                                generation,
                                error,
                                retry_in,
                            } => {
                                if controller.poll_generation != generation {
                                    return;
                                }
                                controller
                                    .record_connection_event(format!("Refresh failed: {error}"));
                                controller.connection = if controller.has_cached_data {
                                    ConnectionState::Stale
                                } else {
                                    ConnectionState::Error(error)
                                };
                                controller.refresh_generation =
                                    controller.refresh_generation.wrapping_add(1);
                                controller.next_refresh_in = retry_in;
                                controller.activity = retry_in
                                    .map(|seconds| format!("Refresh failed · retry in {seconds}s"))
                                    .unwrap_or_else(|| "Refresh failed".into());
                                controller.fetching = false;
                                controller.update_tray(cx);
                            }
                            ConnectResult::HistoryFailure { date, error } => {
                                if controller.history_date == date {
                                    if let Some(previous) = controller.history_previous_date.take()
                                    {
                                        controller.history_date = previous;
                                        controller.history_is_manual = controller.history_date
                                            != chrono::Local::now().date_naive();
                                    }
                                }
                                controller.fetching = false;
                                controller.activity = format!("History unavailable: {error}");
                            }
                            ConnectResult::Stopped { error } => {
                                controller.apply_stopped(error, cx);
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
            let _ = entity.update(cx, |controller, cx| {
                if controller.poll_generation == poll_generation && controller.polling {
                    controller.apply_stopped("polling worker exited unexpectedly".into(), cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(crate) fn stop_polling(&mut self) {
        self.poll_generation = self.poll_generation.wrapping_add(1);
        if let Some(sender) = self.poll_sender.take() {
            let _ = sender.try_send(PollCommand::Stop);
        }
        if let Some(cancel) = self.poll_cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        self.polling = false;
        self.fetching = false;
    }

    pub(crate) fn select_inverter(&mut self, serial: String, cx: &mut Context<Self>) {
        if self.fetching {
            return;
        }
        if self.selected_serial.as_deref() == Some(serial.as_str()) {
            return;
        }
        let plant_id = self
            .inverters
            .iter()
            .find(|inverter| inverter.serial == serial)
            .and_then(|inverter| inverter.plant_id);
        if self.polling
            && !self.send_poll_command(PollCommand::Select(serial.clone(), plant_id), cx)
        {
            self.activity = "Polling is busy; try selecting the inverter again".into();
            cx.notify();
            return;
        }
        self.selected_serial = Some(serial.clone());
        crate::storage::settings::save_selected_serial_async(serial);
        if !self.polling {
            self.activity = "Inverter selected".into();
        }
        cx.notify();
    }

    pub(crate) fn refresh_now(&mut self, cx: &mut Context<Self>) {
        if !self.send_poll_command(PollCommand::Refresh, cx) && !self.fetching {
            self.activity = "Polling is unavailable".into();
            cx.notify();
        }
    }

    pub(crate) fn reconnect(
        &mut self,
        email: String,
        password: String,
        refresh_seconds: u64,
        history_days: u64,
        cx: &mut Context<Self>,
    ) {
        self.refresh_seconds = refresh_seconds.clamp(1, 3600);
        self.history_days = history_days.clamp(1, 3650);
        crate::storage::settings::save_refresh_seconds_async(self.refresh_seconds);
        crate::storage::settings::save_history_days_async(self.history_days);
        self.connect(email, password, cx);
    }

    pub(crate) fn change_history_day(&mut self, offset: i64, cx: &mut Context<Self>) {
        let date = self.history_date + chrono::Duration::days(offset);
        self.select_history_date(date, cx);
    }

    pub(crate) fn select_history_date(&mut self, date: chrono::NaiveDate, cx: &mut Context<Self>) {
        if self.fetching {
            return;
        }
        let today = chrono::Local::now().date_naive();
        if date > today || date == self.history_date {
            return;
        }
        if !self.polling {
            self.activity = "Historical data is unavailable until connected".into();
            cx.notify();
            return;
        }
        self.history_previous_date = Some(self.history_date);
        self.history_date = date;
        self.history_is_manual = date != today;
        if let Some(serial) = self.selected_serial.as_ref() {
            let cached = self.history_cache.get(&(serial.clone(), date)).cloned();
            if let Some((history, source)) = cached {
                self.history_cache_order
                    .retain(|key| key != &(serial.clone(), date));
                self.history_cache_order.push_back((serial.clone(), date));
                self.state.set_history(history);
                self.history_source = source;
                self.history_previous_date = None;
                self.activity = "Historical data loaded from cache".into();
                self.refresh_generation = self.refresh_generation.wrapping_add(1);
                cx.notify();
                return;
            }
        }
        let queued = self
            .poll_sender
            .as_ref()
            .is_some_and(|sender| sender.try_send(PollCommand::HistoryDate(date)).is_ok());
        if queued {
            self.fetching = true;
            self.activity = "Fetching historical data…".into();
        } else {
            self.history_date = self.history_previous_date.take().unwrap_or(today);
            self.activity = "Polling is unavailable".into();
        }
        cx.notify();
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
