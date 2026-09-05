use crate::{
    app::polling::protocol::{Command as PollCommand, PollResult as ConnectResult},
    storage::credentials,
    sunsynk::SunsynkClient,
};
use gpui_kit::*;
use std::{sync::atomic::Ordering, thread};

use super::{ConnectionState, MonitorController};

impl MonitorController {
    pub(crate) fn connect(&mut self, email: String, password: String, cx: &mut Context<Self>) {
        if self.fetching {
            return;
        }
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
        let saved_token = self
            .credentials
            .as_ref()
            .filter(|(saved_email, _)| !credentials_changed && saved_email == &email)
            .and_then(|_| self.refresh_token.clone());
        self.credentials = Some((email.clone(), password.clone()));
        let saved_selection = self.selected_serial.clone();
        let refresh_seconds = self.refresh_seconds;
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let settings = self.state.settings.clone();
        let connect_epoch = self.connect_epoch.clone();
        let progress_sender = sender.clone();
        thread::spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| anyhow::anyhow!(e))
                .and_then(|runtime| {
                    runtime.block_on(async move {
                        let mut client = SunsynkClient::new(
                            settings.api_base_url,
                            email.clone(),
                            password.clone(),
                        )?
                        .with_refresh_token(saved_token)
                        .with_progress(move |message| {
                            let _ = progress_sender.send(ConnectResult::Progress {
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
                        if let Err(error) = credentials::save(
                            &email,
                            &password,
                            client.refresh_token(),
                            selected.as_deref(),
                            refresh_seconds,
                            None,
                        ) {
                            tracing::warn!(%error, "could not save SunSynk credentials");
                        }
                        Ok::<_, anyhow::Error>((
                            inverters,
                            snapshot,
                            selected,
                            client.refresh_token().map(str::to_owned),
                            history,
                        ))
                    })
                });
            let _ = sender.send(match result {
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
            });
        });
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
                            dashboard.fetching = false;
                            dashboard.selected_serial = selected.clone();
                            dashboard.inverters = inverters;
                            dashboard.refresh_token = refresh_token;
                            let has_snapshot = snapshot.is_some();
                            if let Some(snapshot) = snapshot {
                                dashboard.state.set_snapshot(snapshot);
                                dashboard.has_cached_data = true;
                            }
                            if let Some(history) = history {
                                dashboard.state.set_history(history);
                            }
                            dashboard.connection = if has_snapshot {
                                ConnectionState::Connected
                            } else {
                                ConnectionState::Error(
                                    "Account connected, but no live inverter data is available yet."
                                        .into(),
                                )
                            };
                            dashboard.update_tray(cx);
                            dashboard.next_refresh_in = Some(refresh_seconds);
                            dashboard.activity = format!(
                                "Waiting for next refresh · next refresh in {refresh_seconds}s"
                            );
                            dashboard.refresh_seconds = refresh_seconds;
                            if dashboard.polling {
                                if let Some((email, password)) = dashboard.credentials.clone() {
                                    if let Some(serial) = selected.clone() {
                                        dashboard.send_poll_command(
                                            PollCommand::Reconfigure {
                                                base_url: dashboard
                                                    .state
                                                    .settings
                                                    .api_base_url
                                                    .clone(),
                                                email,
                                                password,
                                                serial,
                                                plant_id: dashboard
                                                    .inverters
                                                    .iter()
                                                    .find(|i| {
                                                        i.serial
                                                            == selected
                                                                .as_deref()
                                                                .unwrap_or_default()
                                                    })
                                                    .and_then(|i| i.plant_id),
                                                refresh_token: dashboard.refresh_token.clone(),
                                                interval: refresh_seconds,
                                            },
                                            cx,
                                        );
                                    }
                                }
                            } else {
                                if dashboard.selected_serial.is_some() {
                                    dashboard.start_polling(cx);
                                }
                            }
                        }
                        ConnectResult::PollStarted => {}
                        ConnectResult::Progress { message } => {
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
                            dashboard.state.set_snapshot(snapshot);
                            dashboard.has_cached_data = true;
                            let token_changed = refresh_token != dashboard.refresh_token;
                            dashboard.refresh_token = refresh_token.clone();
                            if let Some(history) = history {
                                dashboard.state.set_history(history);
                            }
                            if token_changed {
                                if let (Some((email, _)), Some(token)) =
                                    (&dashboard.credentials, refresh_token.as_deref())
                                {
                                    credentials::save_refresh_token_async(
                                        email.clone(),
                                        token.to_owned(),
                                    );
                                }
                            }
                            dashboard.connection = ConnectionState::Connected;
                            dashboard.fetching = false;
                            dashboard.update_tray(cx);
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
                            let failure_activity = format!("Login failed: {error}");
                            dashboard.connection = ConnectionState::Error(error);
                            dashboard.inverters.clear();
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
        self.refresh_seconds = refresh_seconds;
        let credentials_changed =
            self.credentials
                .as_ref()
                .is_some_and(|(current_email, current_password)| {
                    current_email != &email || current_password != &password
                });
        if self.polling && !credentials_changed {
            self.refresh_now(cx);
        } else {
            self.connect(email, password, cx);
        }
    }
}
