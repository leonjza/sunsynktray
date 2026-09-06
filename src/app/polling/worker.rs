use super::protocol::{Command, PollConfig, PollResult};
use crate::sunsynk::SunsynkClient;
use anyhow::anyhow;
use futures_util::future::{select, Either};
use std::{thread, time::Duration};

pub(crate) fn spawn(
    config: PollConfig,
) -> (
    tokio::sync::mpsc::Sender<Command>,
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::mpsc::Receiver<PollResult>,
) {
    const COMMAND_CAPACITY: usize = 8;
    const RESULT_CAPACITY: usize = 32;
    let (command_sender, mut command_receiver) = tokio::sync::mpsc::channel(COMMAND_CAPACITY);
    let (cancel_sender, mut cancel_receiver) = tokio::sync::oneshot::channel();
    let (sender, receiver) = tokio::sync::mpsc::channel(RESULT_CAPACITY);

    thread::spawn(move || {
        let generation = config.generation;
        let mut serial = config.serial;
        let mut plant_id = config.plant_id;
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = sender.blocking_send(PollResult::Stopped {
                    error: format!("polling runtime stopped: {error}"),
                });
                return;
            }
        };
        runtime.block_on(async move {
            let interval = config.interval_seconds.max(1);
            let mut retry_delay = interval;
            let mut client =
                match SunsynkClient::new(config.base_url, config.email, config.password) {
                    Ok(client) => client
                        .with_refresh_token(config.refresh_token)
                        .with_progress({
                            let sender = sender.clone();
                            move |message| {
                                let _ = sender.try_send(PollResult::Progress {
                                    generation,
                                    message: message.to_owned(),
                                });
                            }
                        }),
                    Err(error) => {
                        let _ = send_cancellable(
                            &sender,
                            PollResult::Stopped {
                                error: format!("polling client stopped: {error}"),
                            },
                            &mut cancel_receiver,
                        )
                        .await;
                        return;
                    }
                };
            loop {
                match cancellable(
                    tokio::time::timeout(Duration::from_secs(retry_delay), command_receiver.recv()),
                    &mut cancel_receiver,
                )
                .await
                {
                    None => break,
                    Some(Ok(Some(Command::Refresh))) => {}
                    Some(Ok(Some(Command::Stop))) => break,
                    Some(Ok(Some(Command::Select(next_serial, next_plant_id)))) => {
                        serial = next_serial;
                        plant_id = next_plant_id;
                    }
                    Some(Ok(Some(Command::HistoryDate(date)))) => {
                        if !send_cancellable(&sender, PollResult::PollStarted, &mut cancel_receiver)
                            .await
                        {
                            break;
                        }
                        let result = cancellable(
                            async {
                                if let Some(plant_id) = plant_id {
                                    client.history(plant_id, &date.to_string()).await
                                } else {
                                    Err(anyhow!("selected inverter has no plant"))
                                }
                            },
                            &mut cancel_receiver,
                        )
                        .await;
                        let Some(result) = result else { break };
                        let result = result.map(PollResult::History).unwrap_or_else(|error| {
                            PollResult::HistoryFailure {
                                date,
                                error: error.to_string(),
                            }
                        });
                        if !send_cancellable(&sender, result, &mut cancel_receiver).await {
                            break;
                        }
                        continue;
                    }
                    Some(Err(_)) => break,
                    Some(Ok(None)) => break,
                }

                if !send_cancellable(&sender, PollResult::PollStarted, &mut cancel_receiver).await {
                    break;
                }
                let result = cancellable(
                    async {
                        match plant_id {
                            Some(plant_id) => client.refresh_plant(plant_id, &serial).await,
                            None => Err(anyhow!("selected inverter has no plant")),
                        }
                    },
                    &mut cancel_receiver,
                )
                .await;
                let Some(result) = result else { break };
                let succeeded = result.is_ok();
                let retry_in = next_retry_delay(succeeded, interval, retry_delay);
                let result = result
                    .map(|(snapshot, history)| PollResult::Snapshot {
                        snapshot,
                        refresh_token: client.refresh_token().map(str::to_owned),
                        history,
                    })
                    .unwrap_or_else(|error| PollResult::Failure {
                        generation,
                        error: error.to_string(),
                        retry_in: Some(retry_in),
                    });
                if !send_cancellable(&sender, result, &mut cancel_receiver).await {
                    break;
                }
                retry_delay = retry_in;
            }
        });
    });

    (command_sender, cancel_sender, receiver)
}

async fn cancellable<F, T>(future: F, cancel: &mut tokio::sync::oneshot::Receiver<()>) -> Option<T>
where
    F: std::future::Future<Output = T>,
{
    match select(Box::pin(future), Box::pin(&mut *cancel)).await {
        Either::Left((output, _)) => Some(output),
        Either::Right(_) => None,
    }
}

async fn send_cancellable(
    sender: &tokio::sync::mpsc::Sender<PollResult>,
    result: PollResult,
    cancel: &mut tokio::sync::oneshot::Receiver<()>,
) -> bool {
    cancellable(sender.send(result), cancel)
        .await
        .is_some_and(|result| result.is_ok())
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
    use super::{next_retry_delay, send_cancellable, spawn};
    use crate::app::polling::protocol::{Command, PollConfig, PollResult};
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
        let (sender, _cancel, mut receiver) = spawn(PollConfig {
            generation: 1,
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

    #[test]
    fn cancellable_future_stops_when_cancelled() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (sender, mut receiver) = tokio::sync::oneshot::channel();
            sender.send(()).unwrap();
            let result = super::cancellable(std::future::pending::<()>(), &mut receiver).await;
            assert!(result.is_none());
        });
    }

    #[test]
    fn blocked_result_send_stops_when_cancelled() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
            sender.send(PollResult::PollStarted).await.unwrap();
            let (cancel_sender, mut cancel_receiver) = tokio::sync::oneshot::channel();
            cancel_sender.send(()).unwrap();
            assert!(
                !send_cancellable(&sender, PollResult::PollStarted, &mut cancel_receiver,).await
            );
            assert!(receiver.try_recv().is_ok());
        });
    }
}
