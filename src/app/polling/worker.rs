use super::protocol::{Command, PollConfig, PollResult};
use crate::sunsynk::SunsynkClient;
use anyhow::anyhow;
use futures_util::future::{select, Either};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

struct BackfillTaskResult {
    date: chrono::NaiveDate,
    result: anyhow::Result<()>,
}

struct HistoryTaskResult {
    date: chrono::NaiveDate,
    result: anyhow::Result<(Vec<crate::domain::HistorySeries>, crate::app::HistorySource)>,
}

struct LiveTaskResult {
    result: anyhow::Result<(
        crate::domain::EnergySnapshot,
        Option<Vec<crate::domain::HistorySeries>>,
    )>,
    refresh_token: Option<String>,
}

pub(crate) fn spawn(
    config: PollConfig,
) -> (
    tokio::sync::mpsc::Sender<Command>,
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::mpsc::Receiver<PollResult>,
    Option<tokio::task::JoinHandle<()>>,
) {
    const COMMAND_CAPACITY: usize = 8;
    const RESULT_CAPACITY: usize = 32;
    let (command_sender, command_receiver) = tokio::sync::mpsc::channel(COMMAND_CAPACITY);
    let (cancel_sender, cancel_receiver) = tokio::sync::oneshot::channel();
    let (sender, receiver) = tokio::sync::mpsc::channel(RESULT_CAPACITY);

    let task = match crate::app::runtime::spawn(run(
        config,
        command_receiver,
        cancel_receiver,
        sender.clone(),
    )) {
        Ok(task) => Some(task),
        Err(error) => {
            let _ = sender.try_send(PollResult::Stopped {
                error: error.clone(),
            });
            None
        }
    };

    (command_sender, cancel_sender, receiver, task)
}

async fn run(
    config: PollConfig,
    mut command_receiver: tokio::sync::mpsc::Receiver<Command>,
    mut cancel_receiver: tokio::sync::oneshot::Receiver<()>,
    sender: tokio::sync::mpsc::Sender<PollResult>,
) {
    const BACKFILL_INTERVAL: Duration = Duration::from_secs(15);
    let generation = config.generation;
    let mut serial = config.serial;
    let mut plant_id = config.plant_id;
    let interval = config.interval_seconds.max(1);
    let mut retry_delay = interval;
    let mut next_refresh_at = Instant::now();
    let mut next_today_history_at = Instant::now();
    let total_backfill = config.history_days.saturating_sub(1);
    let today = chrono::Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let mut oldest_date = today - chrono::Duration::days(total_backfill as i64);
    let mut backfill_day = today;
    let saved_backfill = match config
        .database
        .load_backfill_state(config.email.clone(), serial.clone())
    {
        Ok(state) => state,
        Err(error) => {
            tracing::warn!(%error, %serial, "could not load SQLite backfill state");
            None
        }
    };
    let mut backfill_date = saved_backfill
        .as_ref()
        .and_then(|state| state.target_date.parse::<chrono::NaiveDate>().ok())
        .map(|date| {
            if total_backfill == 0 {
                yesterday
            } else {
                date.clamp(oldest_date, yesterday)
            }
        })
        .unwrap_or(yesterday);
    let mut backfill_completed = yesterday
        .signed_duration_since(backfill_date)
        .num_days()
        .max(0) as u64;
    let mut backfill_running = saved_backfill
        .as_ref()
        .is_none_or(|state| state.status != "paused")
        && backfill_completed < total_backfill;
    let mut next_backfill_at = Instant::now();
    let mut backfill_task: Option<tokio::task::JoinHandle<BackfillTaskResult>> = None;
    let mut history_task: Option<tokio::task::JoinHandle<HistoryTaskResult>> = None;
    let mut live_task: Option<tokio::task::JoinHandle<LiveTaskResult>> = None;
    let backfill_cancel_epoch = Arc::new(AtomicU64::new(0));
    let initial_detail = saved_backfill
        .as_ref()
        .and_then(|state| state.last_error.clone())
        .map(|error| format!("Previous attempt failed: {error}"));
    let save_backfill = |current_serial: &str,
                         date: chrono::NaiveDate,
                         oldest: chrono::NaiveDate,
                         status: &str,
                         error: Option<String>| {
        config.database.save_backfill_state(
            config.email.clone(),
            current_serial.to_owned(),
            date.to_string(),
            oldest.to_string(),
            status.to_owned(),
            error,
        );
    };
    save_backfill(
        &serial,
        backfill_date,
        oldest_date,
        if backfill_running {
            "running"
        } else {
            "paused"
        },
        None,
    );
    let client = match SunsynkClient::new(
        config.base_url,
        config.email.clone(),
        config.password.clone(),
    ) {
        Ok(client) => client
            .with_auth_state(config.auth)
            .with_progress({
                let sender = sender.clone();
                move |message| {
                    let _ = sender.try_send(PollResult::Progress {
                        generation,
                        message: message.to_owned(),
                    });
                }
            })
            .with_request_log({
                let connection_log = config.connection_log.clone();
                let connection_log_revision = config.connection_log_revision.clone();
                let connection_log_signal = config.connection_log_signal.clone();
                move |message| {
                    crate::app::MonitorController::record_connection_event_to(
                        &connection_log,
                        &connection_log_revision,
                        &connection_log_signal,
                        message,
                    );
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
    if !send_cancellable(
        &sender,
        PollResult::BackfillProgress {
            completed: backfill_completed,
            total: total_backfill,
            running: backfill_running,
            detail: if let Some(detail) = initial_detail {
                detail
            } else if backfill_running {
                format!("Next: {backfill_date}")
            } else {
                "Paused".into()
            },
            next_request_in: None,
        },
        &mut cancel_receiver,
    )
    .await
    {
        return;
    }
    loop {
        if live_task.as_ref().is_some_and(|task| task.is_finished()) {
            if let Some(task) = live_task.take() {
                let LiveTaskResult {
                    result,
                    refresh_token,
                } = match task.await {
                    Ok(result) => result,
                    Err(error) => LiveTaskResult {
                        result: Err(anyhow!("live refresh task failed: {error}")),
                        refresh_token: None,
                    },
                };
                let succeeded = result.is_ok();
                let retry_in = next_retry_delay(succeeded, interval, retry_delay);
                let result = result
                    .map(|(snapshot, history)| PollResult::Snapshot {
                        snapshot,
                        refresh_token,
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
                next_refresh_at = Instant::now() + Duration::from_secs(retry_in);
            }
        }
        if history_task.as_ref().is_some_and(|task| task.is_finished()) {
            if let Some(task) = history_task.take() {
                match task.await {
                    Ok(HistoryTaskResult {
                        date,
                        result: Ok((history, source)),
                    }) => {
                        if !history.is_empty() {
                            config.database.save_history(
                                config.email.clone(),
                                serial.clone(),
                                date.to_string(),
                                history.clone(),
                            );
                        }
                        if !send_cancellable(
                            &sender,
                            PollResult::History { history, source },
                            &mut cancel_receiver,
                        )
                        .await
                        {
                            break;
                        }
                    }
                    Ok(HistoryTaskResult {
                        date,
                        result: Err(error),
                    }) => {
                        if !send_cancellable(
                            &sender,
                            PollResult::HistoryFailure {
                                date,
                                error: error.to_string(),
                            },
                            &mut cancel_receiver,
                        )
                        .await
                        {
                            break;
                        }
                    }
                    Err(error) => tracing::warn!(%error, "history task stopped unexpectedly"),
                }
            }
        }
        if backfill_task
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            if let Some(task) = backfill_task.take() {
                match task.await {
                    Ok(BackfillTaskResult {
                        date,
                        result: Ok(()),
                    }) if date == backfill_date => {
                        backfill_completed += 1;
                        backfill_date = date - chrono::Duration::days(1);
                        let complete = backfill_completed >= total_backfill;
                        backfill_running = !complete;
                        next_backfill_at = if complete {
                            Instant::now()
                        } else {
                            Instant::now() + BACKFILL_INTERVAL
                        };
                        save_backfill(
                            &serial,
                            backfill_date,
                            oldest_date,
                            if complete { "complete" } else { "running" },
                            None,
                        );
                        if !send_cancellable(
                            &sender,
                            PollResult::BackfillProgress {
                                completed: backfill_completed,
                                total: total_backfill,
                                running: true,
                                detail: if complete {
                                    "Up to date".into()
                                } else {
                                    format!("Processed {date}")
                                },
                                next_request_in: backfill_running
                                    .then(|| backfill_wait_seconds(next_backfill_at)),
                            },
                            &mut cancel_receiver,
                        )
                        .await
                        {
                            break;
                        }
                    }
                    Ok(BackfillTaskResult {
                        date,
                        result: Err(error),
                    }) => {
                        let transport_error = is_transport_error(&error);
                        next_backfill_at = Instant::now() + BACKFILL_INTERVAL;
                        if !transport_error {
                            backfill_completed += 1;
                            backfill_date = date - chrono::Duration::days(1);
                        }
                        if !transport_error && backfill_completed >= total_backfill {
                            backfill_running = false;
                        }
                        let complete = !backfill_running;
                        save_backfill(
                            &serial,
                            if transport_error { date } else { backfill_date },
                            oldest_date,
                            if complete { "complete" } else { "running" },
                            transport_error.then(|| error.to_string()),
                        );
                        if !send_cancellable(
                            &sender,
                            PollResult::BackfillProgress {
                                completed: backfill_completed,
                                total: total_backfill,
                                running: true,
                                detail: if complete {
                                    "Up to date".into()
                                } else if transport_error {
                                    format!("Failed {date}: {error}; will retry")
                                } else {
                                    format!("Skipped {date}: {error}")
                                },
                                next_request_in: backfill_running
                                    .then(|| backfill_wait_seconds(next_backfill_at)),
                            },
                            &mut cancel_receiver,
                        )
                        .await
                        {
                            break;
                        }
                    }
                    Ok(_) => {}
                    Err(error) => tracing::warn!(%error, "backfill task stopped unexpectedly"),
                }
            }
        }
        let current_today = chrono::Local::now().date_naive();
        if current_today != backfill_day {
            backfill_cancel_epoch.fetch_add(1, Ordering::Release);
            backfill_day = current_today;
            let yesterday = current_today - chrono::Duration::days(1);
            oldest_date = current_today - chrono::Duration::days(total_backfill as i64);
            backfill_date = yesterday;
            backfill_completed = 0;
            next_backfill_at = Instant::now();
            next_today_history_at = Instant::now();
            save_backfill(
                &serial,
                backfill_date,
                oldest_date,
                if backfill_running {
                    "running"
                } else {
                    "paused"
                },
                None,
            );
        }
        let refresh_wait = next_refresh_at.saturating_duration_since(Instant::now());
        let wait = if backfill_task.is_some() {
            refresh_wait.min(Duration::from_secs(1))
        } else {
            refresh_wait
        };
        let wait = if live_task.is_some() || history_task.is_some() {
            wait.min(Duration::from_secs(1))
        } else {
            wait
        };
        let backfill_wait = if backfill_running && backfill_task.is_none() {
            next_backfill_at.saturating_duration_since(Instant::now())
        } else {
            Duration::from_secs(1)
        };
        let wait = wait.min(backfill_wait);
        match cancellable(
            tokio::time::timeout(wait, command_receiver.recv()),
            &mut cancel_receiver,
        )
        .await
        {
            None => break,
            Some(Ok(Some(Command::Refresh))) => {
                // Make the scheduled-refresh path pick this up immediately.
                next_refresh_at = Instant::now();
            }
            Some(Ok(Some(Command::Stop))) => {
                backfill_cancel_epoch.fetch_add(1, Ordering::Release);
                if let Some(task) = backfill_task.take() {
                    task.abort();
                }
                if let Some(task) = live_task.take() {
                    task.abort();
                }
                if let Some(task) = history_task.take() {
                    task.abort();
                }
                break;
            }
            Some(Ok(Some(Command::Select(next_serial, next_plant_id)))) => {
                backfill_cancel_epoch.fetch_add(1, Ordering::Release);
                if let Some(task) = backfill_task.take() {
                    task.abort();
                }
                if let Some(task) = live_task.take() {
                    task.abort();
                }
                if let Some(task) = history_task.take() {
                    task.abort();
                }
                serial = next_serial;
                plant_id = next_plant_id;
                let saved = match config
                    .database
                    .load_backfill_state(config.email.clone(), serial.clone())
                {
                    Ok(state) => state,
                    Err(error) => {
                        tracing::warn!(%error, %serial, "could not load SQLite backfill state");
                        None
                    }
                };
                let today = chrono::Local::now().date_naive();
                let yesterday = today - chrono::Duration::days(1);
                oldest_date = today - chrono::Duration::days(total_backfill as i64);
                backfill_date = saved
                    .as_ref()
                    .and_then(|state| state.target_date.parse::<chrono::NaiveDate>().ok())
                    .map(|date| {
                        if total_backfill == 0 {
                            yesterday
                        } else {
                            date.clamp(oldest_date, yesterday)
                        }
                    })
                    .unwrap_or(yesterday);
                backfill_completed = yesterday
                    .signed_duration_since(backfill_date)
                    .num_days()
                    .max(0) as u64;
                next_backfill_at = Instant::now();
                next_today_history_at = Instant::now();
                backfill_running = saved.as_ref().is_none_or(|state| state.status != "paused")
                    && backfill_completed < total_backfill;
                save_backfill(
                    &serial,
                    backfill_date,
                    oldest_date,
                    if backfill_running {
                        "running"
                    } else {
                        "paused"
                    },
                    None,
                );
                if !send_cancellable(
                    &sender,
                    PollResult::BackfillProgress {
                        completed: backfill_completed,
                        total: total_backfill,
                        running: backfill_running,
                        detail: if backfill_running {
                            format!("Next: {backfill_date}")
                        } else {
                            "Paused".into()
                        },
                        next_request_in: backfill_running
                            .then(|| backfill_wait_seconds(next_backfill_at)),
                    },
                    &mut cancel_receiver,
                )
                .await
                {
                    break;
                }
                next_refresh_at = Instant::now();
            }
            Some(Ok(Some(Command::PauseBackfill))) => {
                backfill_cancel_epoch.fetch_add(1, Ordering::Release);
                backfill_running = false;
                next_backfill_at = Instant::now() + BACKFILL_INTERVAL;
                if let Some(task) = backfill_task.take() {
                    task.abort();
                }
                save_backfill(&serial, backfill_date, oldest_date, "paused", None);
                let _ = send_cancellable(
                    &sender,
                    PollResult::BackfillProgress {
                        completed: backfill_completed,
                        total: total_backfill,
                        running: false,
                        detail: "Paused".into(),
                        next_request_in: None,
                    },
                    &mut cancel_receiver,
                )
                .await;
                continue;
            }
            Some(Ok(Some(Command::ResumeBackfill))) => {
                backfill_running = true;
                next_backfill_at = Instant::now();
                save_backfill(&serial, backfill_date, oldest_date, "running", None);
                let _ = send_cancellable(
                    &sender,
                    PollResult::BackfillProgress {
                        completed: backfill_completed,
                        total: total_backfill,
                        running: true,
                        detail: format!("Processing {backfill_date}"),
                        next_request_in: Some(backfill_wait_seconds(next_backfill_at)),
                    },
                    &mut cancel_receiver,
                )
                .await;
                continue;
            }
            Some(Ok(Some(Command::HistoryDate(date)))) => {
                if let Some(task) = history_task.take() {
                    task.abort();
                }
                if !send_cancellable(&sender, PollResult::PollStarted, &mut cancel_receiver).await {
                    break;
                }
                let database = config.database.clone();
                let email = config.email.clone();
                let serial_for_history = serial.clone();
                let history_client = client.clone();
                let connection_log = config.connection_log.clone();
                let connection_log_revision = config.connection_log_revision.clone();
                let connection_log_signal = config.connection_log_signal.clone();
                history_task = Some(tokio::spawn(async move {
                    let result = if let Some(plant_id) = plant_id {
                        let cached_result = tokio::task::spawn_blocking({
                            let database = database.clone();
                            let email = email.clone();
                            let serial = serial_for_history.clone();
                            let date = date.to_string();
                            move || database.load_history(email, Some(serial), date)
                        })
                        .await;
                        let cached = match cached_result {
                            Ok(Ok(history)) => history,
                            Ok(Err(error)) => {
                                crate::app::MonitorController::record_connection_event_to(
                                    &connection_log,
                                    &connection_log_revision,
                                    &connection_log_signal,
                                    format!("SQLite history read failed: {error}"),
                                );
                                Vec::new()
                            }
                            Err(error) => {
                                crate::app::MonitorController::record_connection_event_to(
                                    &connection_log,
                                    &connection_log_revision,
                                    &connection_log_signal,
                                    format!("SQLite history task failed: {error}"),
                                );
                                Vec::new()
                            }
                        };
                        if cached.is_empty() {
                            history_client
                                .history(plant_id, &date.to_string())
                                .await
                                .map(|history| (history, crate::app::HistorySource::Network))
                        } else {
                            Ok((cached, crate::app::HistorySource::Cached))
                        }
                    } else {
                        Err(anyhow!("selected inverter has no plant"))
                    };
                    HistoryTaskResult { date, result }
                }));
                continue;
            }
            Some(Err(_)) => {
                let now = Instant::now();
                let refresh_due = now >= next_refresh_at;
                let backfill_due =
                    backfill_running && backfill_task.is_none() && now >= next_backfill_at;
                if !refresh_due && !backfill_due {
                    continue;
                }
            }
            Some(Ok(None)) => break,
        }

        if backfill_running
            && backfill_completed < total_backfill
            && backfill_task.is_none()
            && Instant::now() >= next_backfill_at
        {
            let Some(plant_id) = plant_id else {
                next_backfill_at = Instant::now() + BACKFILL_INTERVAL;
                tracing::warn!(%serial, "cannot backfill inverter without a plant");
                let _ = send_cancellable(
                    &sender,
                    PollResult::BackfillProgress {
                        completed: backfill_completed,
                        total: total_backfill,
                        running: true,
                        detail: format!("Waiting for plant information · next: {backfill_date}"),
                        next_request_in: Some(backfill_wait_seconds(next_backfill_at)),
                    },
                    &mut cancel_receiver,
                )
                .await;
                continue;
            };
            let date = backfill_date;
            if !send_cancellable(
                &sender,
                PollResult::BackfillProgress {
                    completed: backfill_completed,
                    total: total_backfill,
                    running: true,
                    detail: format!("Processing {date}"),
                    next_request_in: None,
                },
                &mut cancel_receiver,
            )
            .await
            {
                break;
            }
            let history_client = client.clone();
            let database = config.database.clone();
            let email = config.email.clone();
            let serial_for_task = serial.clone();
            let oldest_for_task = oldest_date;
            let cancel_epoch = backfill_cancel_epoch.clone();
            let expected_epoch = cancel_epoch.load(Ordering::Acquire);
            backfill_task = Some(tokio::spawn(async move {
                let result = match history_client.history(plant_id, &date.to_string()).await {
                    Ok(history) => tokio::task::spawn_blocking(move || {
                        database.save_history_and_backfill(
                            email,
                            serial_for_task,
                            date.to_string(),
                            history,
                            (date - chrono::Duration::days(1)).to_string(),
                            oldest_for_task.to_string(),
                            "running".into(),
                            cancel_epoch,
                            expected_epoch,
                        )
                    })
                    .await
                    .unwrap_or_else(|error| Err(anyhow!("database task failed: {error}"))),
                    Err(error) => Err(error),
                };
                BackfillTaskResult { date, result }
            }));
        }

        if live_task.is_none() && Instant::now() >= next_refresh_at {
            if !send_cancellable(&sender, PollResult::PollStarted, &mut cancel_receiver).await {
                break;
            }
            let live_client = client.clone();
            let live_serial = serial.clone();
            let include_today_history = Instant::now() >= next_today_history_at;
            live_task = Some(tokio::spawn(async move {
                let result = match plant_id {
                    Some(plant_id) => {
                        live_client
                            .refresh_plant(plant_id, &live_serial, include_today_history)
                            .await
                    }
                    None => Err(anyhow!("selected inverter has no plant")),
                };
                LiveTaskResult {
                    result,
                    refresh_token: live_client.refresh_token(),
                }
            }));
            next_refresh_at = Instant::now() + Duration::from_secs(interval);
            if include_today_history {
                next_today_history_at = Instant::now() + Duration::from_secs(300);
            }
        }
    }
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

fn backfill_wait_seconds(next_request_at: Instant) -> u64 {
    let remaining = next_request_at.saturating_duration_since(Instant::now());
    remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0)
}

fn is_transport_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.downcast_ref::<reqwest::Error>().is_some())
}

#[cfg(test)]
mod tests {
    use super::{is_transport_error, next_retry_delay, send_cancellable};
    use crate::app::polling::protocol::{Command, PollConfig, PollResult};
    use anyhow::anyhow;
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
    fn non_transport_errors_are_not_retryable() {
        assert!(!is_transport_error(&anyhow!("no history data returned")));
    }

    #[test]
    fn worker_stops_when_stop_command_is_received() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (sender, command_receiver) = tokio::sync::mpsc::channel(8);
            let (_cancel_sender, cancel_receiver) = tokio::sync::oneshot::channel();
            let (result_sender, mut result_receiver) = tokio::sync::mpsc::channel(32);
            sender.try_send(Command::Stop).unwrap();
            tokio::spawn(super::run(
                PollConfig {
                    generation: 1,
                    base_url: "http://127.0.0.1:1".into(),
                    email: "test@example.com".into(),
                    password: "password".into(),
                    auth: std::sync::Arc::new(std::sync::Mutex::new(
                        crate::sunsynk::AuthState::default(),
                    )),
                    serial: "serial".into(),
                    plant_id: None,
                    interval_seconds: 3600,
                    history_days: 365,
                    database: crate::storage::database::Database::open().unwrap(),
                    connection_log: std::sync::Arc::new(std::sync::Mutex::new(
                        std::collections::VecDeque::new(),
                    )),
                    connection_log_revision: std::sync::Arc::new(
                        std::sync::atomic::AtomicU64::new(0),
                    ),
                    connection_log_signal: std::sync::Arc::new(tokio::sync::Notify::new()),
                },
                command_receiver,
                cancel_receiver,
                result_sender,
            ));
            assert!(matches!(
                result_receiver.recv().await,
                Some(PollResult::BackfillProgress { .. })
            ));
            assert!(
                tokio::time::timeout(Duration::from_secs(2), result_receiver.recv())
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
