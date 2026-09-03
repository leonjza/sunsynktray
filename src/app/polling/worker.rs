use super::protocol::{Command, PollConfig, PollResult};
use crate::sunsynk::SunsynkClient;
use anyhow::anyhow;
use std::{thread, time::Duration};

pub(crate) fn spawn(
    config: PollConfig,
) -> (
    tokio::sync::mpsc::Sender<Command>,
    tokio::sync::mpsc::UnboundedReceiver<PollResult>,
) {
    const COMMAND_CAPACITY: usize = 8;
    let (command_sender, mut command_receiver) = tokio::sync::mpsc::channel(COMMAND_CAPACITY);
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    thread::spawn(move || {
        let mut serial = config.serial;
        let mut plant_id = config.plant_id;
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = sender.send(PollResult::Stopped {
                    error: format!("polling runtime stopped: {error}"),
                });
                return;
            }
        };
        runtime.block_on(async move {
            let mut interval = config.interval_seconds.max(1);
            let mut retry_delay = interval;
            let mut client =
                match SunsynkClient::new(config.base_url, config.email, config.password) {
                    Ok(client) => client
                        .with_refresh_token(config.refresh_token)
                        .with_progress({
                            let sender = sender.clone();
                            move |message| {
                                let _ = sender.send(PollResult::Progress {
                                    message: message.to_owned(),
                                });
                            }
                        }),
                    Err(error) => {
                        let _ = sender.send(PollResult::Stopped {
                            error: format!("polling client stopped: {error}"),
                        });
                        return;
                    }
                };
            loop {
                match tokio::time::timeout(
                    Duration::from_secs(retry_delay),
                    command_receiver.recv(),
                )
                .await
                {
                    Ok(Some(Command::Refresh)) => {}
                    Ok(Some(Command::Stop)) => break,
                    Ok(Some(Command::Select(next_serial, next_plant_id))) => {
                        serial = next_serial;
                        plant_id = next_plant_id;
                    }
                    Ok(Some(Command::Reconfigure {
                        base_url,
                        email,
                        password,
                        serial: next_serial,
                        plant_id: next_plant_id,
                        refresh_token,
                        interval: next_interval,
                    })) => {
                        client = match SunsynkClient::new(base_url, email, password) {
                            Ok(client) => client.with_refresh_token(refresh_token).with_progress({
                                let sender = sender.clone();
                                move |message| {
                                    let _ = sender.send(PollResult::Progress {
                                        message: message.to_owned(),
                                    });
                                }
                            }),
                            Err(error) => {
                                let _ = sender.send(PollResult::Failure {
                                    generation: 0,
                                    error: error.to_string(),
                                    retry_in: None,
                                });
                                continue;
                            }
                        };
                        serial = next_serial;
                        plant_id = next_plant_id;
                        interval = next_interval.max(1);
                        retry_delay = interval;
                        continue;
                    }
                    Ok(Some(Command::HistoryDate(date))) => {
                        if sender.send(PollResult::PollStarted).is_err() {
                            break;
                        }
                        let result = if let Some(plant_id) = plant_id {
                            client.history(plant_id, &date.to_string()).await
                        } else {
                            Err(anyhow!("selected inverter has no plant"))
                        };
                        let result = result.map(PollResult::History).unwrap_or_else(|error| {
                            PollResult::HistoryFailure {
                                date,
                                error: error.to_string(),
                            }
                        });
                        if sender.send(result).is_err() {
                            break;
                        }
                        continue;
                    }
                    Ok(None) => break,
                    Err(_) => {}
                }

                if sender.send(PollResult::PollStarted).is_err() {
                    break;
                }
                let result = match plant_id {
                    Some(plant_id) => client.refresh_plant(plant_id, &serial).await,
                    None => Err(anyhow!("selected inverter has no plant")),
                };
                let succeeded = result.is_ok();
                let retry_in = next_retry_delay(succeeded, interval, retry_delay);
                let result = result
                    .map(|(snapshot, history)| PollResult::Snapshot {
                        snapshot,
                        refresh_token: client.refresh_token().map(str::to_owned),
                        history,
                    })
                    .unwrap_or_else(|error| PollResult::Failure {
                        generation: 0,
                        error: error.to_string(),
                        retry_in: Some(retry_in),
                    });
                if sender.send(result).is_err() {
                    break;
                }
                retry_delay = retry_in;
            }
        });
    });

    (command_sender, receiver)
}

pub(super) fn next_retry_delay(success: bool, interval: u64, previous: u64) -> u64 {
    if success {
        interval.max(1)
    } else {
        previous.saturating_mul(2).clamp(1, 300)
    }
}

#[cfg(test)]
mod tests {
    use super::{next_retry_delay, spawn};
    use crate::app::polling::protocol::{Command, PollConfig};
    use std::time::Duration;

    #[test]
    fn successful_polls_reset_to_configured_interval() {
        assert_eq!(next_retry_delay(true, 60, 240), 60);
        assert_eq!(next_retry_delay(true, 0, 0), 1);
    }

    #[test]
    fn failed_polls_back_off_and_cap_at_five_minutes() {
        assert_eq!(next_retry_delay(false, 60, 60), 120);
        assert_eq!(next_retry_delay(false, 60, 200), 300);
        assert_eq!(next_retry_delay(false, 60, 300), 300);
    }

    #[test]
    fn worker_stops_when_stop_command_is_received() {
        let (sender, mut receiver) = spawn(PollConfig {
            base_url: "http://127.0.0.1:1".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            serial: "serial".into(),
            plant_id: None,
            refresh_token: None,
            interval_seconds: 3600,
        });
        sender.try_send(Command::Stop).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            assert!(
                tokio::time::timeout(Duration::from_secs(2), receiver.recv())
                    .await
                    .unwrap()
                    .is_none()
            );
        });
    }
}
