use crate::{
    app::polling::protocol::{Command as PollCommand, PollResult as ConnectResult},
    storage::credentials,
    sunsynk::SunsynkClient,
};
use futures_util::future::{select, Either};
use gpui_kit::*;
use std::sync::atomic::Ordering;

use super::{ConnectionState, MonitorController, TrayMetric};

impl MonitorController {
    pub(crate) fn connect(&mut self, email: String, password: String, cx: &mut Context<Self>) {
        if email.trim().is_empty() || password.is_empty() {
            self.connection = ConnectionState::Error("Enter your email and password first.".into());
            cx.notify();
            return;
        }
        if self.polling {
            self.stop_polling();
        }
        if !self.has_cached_data {
            self.connection = ConnectionState::Connecting;
        }
        self.connect_generation = self.connect_generation.wrapping_add(1);
        let generation = self.connect_generation;
        if let Some(cancel) = self.connect_cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(task) = self.connect_task.take() {
            task.abort();
        }
        self.connect_epoch.store(generation, Ordering::SeqCst);
        self.activity = "Logging in…".into();
        self.next_refresh_in = None;
        self.fetching = true;
        let credentials_changed =
            self.credentials
                .as_ref()
                .is_some_and(|(current_email, current_password)| {
                    current_email != &email || current_password != &password
                });
        if credentials_changed {
            self.refresh_token = None;
            self.state.clear_cached_data();
            self.has_cached_data = false;
            self.inverters.clear();
            self.selected_serial = None;
            self.history_date = chrono::Local::now().date_naive();
            self.history_is_manual = false;
            self.history_previous_date = None;
            self.connection = ConnectionState::Connecting;
            self.update_tray(cx);
        }
        let saved_token = self
            .credentials
            .as_ref()
            .filter(|(saved_email, _)| !credentials_changed && saved_email == &email)
            .and_then(|_| self.refresh_token.clone());
        self.credentials = Some((email.clone(), password.clone()));
        let saved_selection = self.selected_serial.clone();
        let refresh_seconds = self.refresh_seconds;
        let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
        let settings = self.state.settings.clone();
        let connect_epoch = self.connect_epoch.clone();
        let progress_sender = sender.clone();
        let (cancel_sender, cancel_receiver) = tokio::sync::oneshot::channel();
        self.connect_cancel = Some(cancel_sender);
        let operation = async move {
            let mut client =
                SunsynkClient::new(settings.api_base_url, email.clone(), password.clone())?
                    .with_refresh_token(saved_token)
                    .with_progress(move |message| {
                        let _ = progress_sender.try_send(ConnectResult::Progress {
                            generation,
                            message: message.to_owned(),
                        });
                    });
            let inverters = client.list_inverters().await?;
            let selected = saved_selection
                .and_then(|serial| {
                    inverters
                        .iter()
                        .find(|i| i.serial == serial)
                        .map(|i| i.serial.clone())
                })
                .or_else(|| {
                    inverters
                        .first()
                        .filter(|i| !i.serial.is_empty())
                        .map(|i| i.serial.clone())
                });
            let selected_plant_id = selected.as_ref().and_then(|serial| {
                inverters
                    .iter()
                    .find(|inverter| &inverter.serial == serial)
                    .and_then(|inverter| inverter.plant_id)
            });
            let plant_data = match (selected.as_deref(), selected_plant_id) {
                (Some(serial), Some(plant_id)) => {
                    Some(client.refresh_plant(plant_id, serial).await?)
                }
                _ => None,
            };
            let (snapshot, history) = plant_data
                .map(|(snapshot, history)| (Some(snapshot), history))
                .unwrap_or((None, None));
            if connect_epoch.load(Ordering::SeqCst) != generation {
                return Err(anyhow::anyhow!("login superseded by a newer attempt"));
            }
            Ok::<_, anyhow::Error>((
                inverters,
                snapshot,
                selected,
                client.refresh_token().map(str::to_owned),
                history,
            ))
        };
        let task_sender = sender.clone();
        match crate::app::runtime::spawn(async move {
            tokio::pin!(operation);
            let result = match select(Box::pin(operation), Box::pin(cancel_receiver)).await {
                Either::Left((result, _)) => result,
                Either::Right(_) => Err(anyhow::anyhow!("login superseded by a newer attempt")),
            };
            let _ = task_sender
                .send(match result {
                    Ok((inverters, snapshot, selected, refresh_token, history)) => {
                        ConnectResult::Connected {
                            generation,
                            inverters,
                            snapshot,
                            selected_serial: selected,
                            refresh_token,
                            history,
                        }
                    }
                    Err(error) => ConnectResult::Failure {
                        generation,
                        error: error.to_string(),
                        retry_in: None,
                    },
                })
                .await;
        }) {
            Ok(task) => self.connect_task = Some(task),
            Err(error) => {
                let _ = sender.try_send(ConnectResult::Failure {
                    generation,
                    error: error.clone(),
                    retry_in: None,
                });
            }
        }
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            while let Some(result) = receiver.recv().await {
                entity.update(cx, |dashboard, cx| {
                    match result {
                        ConnectResult::Connected {
                            generation,
                            inverters,
                            snapshot,
                            selected_serial: selected,
                            refresh_token,
                            history,
                        } => {
                            if generation != dashboard.connect_generation {
                                return;
                            }
                            dashboard.connect_cancel = None;
                            dashboard.connect_task = None;
                            dashboard.fetching = false;
                            dashboard.selected_serial = selected.clone();
                            dashboard.inverters = inverters;
                            dashboard.refresh_token = refresh_token.clone();
                            if let Some((email, password)) = dashboard.credentials.clone() {
                                credentials::save_async(
                                    email,
                                    password,
                                    dashboard.refresh_token.clone(),
                                    selected.clone(),
                                    refresh_seconds,
                                    dashboard
                                        .tray_metric
                                        .map(TrayMetric::saved_name)
                                        .map(str::to_owned),
                                );
                            }
                            let has_snapshot = snapshot.is_some();
                            if let Some(snapshot) = snapshot {
                                dashboard.apply_live_data(snapshot, refresh_token, history, cx);
                            } else if !dashboard.history_is_manual {
                                if let Some(history) = history {
                                    dashboard.state.set_history(history);
                                }
                            }
                            dashboard.connection = if has_snapshot {
                                ConnectionState::Connected
                            } else if dashboard.has_cached_data {
                                ConnectionState::Stale
                            } else {
                                ConnectionState::Error(
                                    "Account connected, but no live inverter data is available yet."
                                        .into(),
                                )
                            };
                            dashboard.refresh_generation =
                                dashboard.refresh_generation.wrapping_add(1);
                            dashboard.next_refresh_in = Some(refresh_seconds);
                            dashboard.activity = format!(
                                "Waiting for next refresh · next refresh in {refresh_seconds}s"
                            );
                            dashboard.refresh_seconds = refresh_seconds;
                            if dashboard.selected_serial.is_some() {
                                dashboard.start_polling(cx);
                            }
                        }
                        ConnectResult::PollStarted => {}
                        ConnectResult::Progress {
                            generation,
                            message,
                        } => {
                            if generation != dashboard.connect_generation {
                                return;
                            }
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
                            dashboard.apply_live_data(snapshot, refresh_token, history, cx);
                            dashboard.fetching = false;
                            dashboard.refresh_generation =
                                dashboard.refresh_generation.wrapping_add(1);
                            dashboard.next_refresh_in = Some(dashboard.refresh_seconds);
                            dashboard.activity = "Waiting for next refresh".into();
                        }
                        ConnectResult::HistoryFailure { error, .. } => {
                            dashboard.fetching = false;
                            dashboard.activity = format!("History unavailable: {error}");
                        }
                        ConnectResult::Failure {
                            generation: result_generation,
                            error,
                            ..
                        } => {
                            if result_generation != dashboard.connect_generation {
                                return;
                            }
                            dashboard.connect_cancel = None;
                            dashboard.connect_task = None;
                            let failure_activity = format!("Login failed: {error}");
                            dashboard.connection = if dashboard.has_cached_data {
                                ConnectionState::Stale
                            } else {
                                ConnectionState::Error(error.clone())
                            };
                            dashboard.next_refresh_in = None;
                            dashboard.activity = failure_activity;
                            dashboard.fetching = false;
                        }
                        ConnectResult::Stopped { error } => {
                            dashboard.apply_stopped(error, cx);
                        }
                    }
                    cx.notify();
                });
            }
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn reconnect_or_connect(
        &mut self,
        email: String,
        password: String,
        refresh_seconds: u64,
        cx: &mut Context<Self>,
    ) {
        let refresh_seconds = refresh_seconds.clamp(1, 3600);
        let interval_changed = self.refresh_seconds != refresh_seconds;
        self.refresh_seconds = refresh_seconds;
        let credentials_changed =
            self.credentials
                .as_ref()
                .is_some_and(|(current_email, current_password)| {
                    current_email != &email || current_password != &password
                });
        if self.polling && !credentials_changed {
            if interval_changed {
                self.stop_polling();
                self.start_polling(cx);
                if self.send_poll_command(PollCommand::Refresh, cx) {
                    if let Some((email, _)) = self.credentials.clone() {
                        credentials::save_refresh_seconds_async(email, refresh_seconds);
                    }
                }
            } else {
                self.refresh_now(cx);
            }
        } else {
            self.connect(email, password, cx);
        }
    }
}
