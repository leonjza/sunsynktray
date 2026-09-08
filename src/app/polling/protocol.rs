use crate::domain::{EnergySnapshot, HistorySeries, InverterSummary};

pub(crate) enum PollResult {
    Connected {
        generation: u64,
        inverters: Vec<InverterSummary>,
        snapshot: Option<EnergySnapshot>,
        selected_serial: Option<String>,
        refresh_token: Option<String>,
        auth: std::sync::Arc<std::sync::Mutex<crate::sunsynk::AuthState>>,
        history: Option<Vec<HistorySeries>>,
    },
    PollStarted,
    Progress {
        generation: u64,
        message: String,
    },
    History {
        history: Vec<HistorySeries>,
        source: crate::app::HistorySource,
    },
    HistoryFailure {
        date: chrono::NaiveDate,
        error: String,
    },
    Snapshot {
        snapshot: EnergySnapshot,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
    },
    BackfillProgress {
        completed: u64,
        total: u64,
        running: bool,
        detail: String,
        next_request_in: Option<u64>,
    },
    Failure {
        generation: u64,
        error: String,
        retry_in: Option<u64>,
    },
    Stopped {
        error: String,
    },
}

pub(crate) enum Command {
    Refresh,
    Stop,
    Select(String, Option<i64>),
    HistoryDate(chrono::NaiveDate),
    PauseBackfill,
    ResumeBackfill,
}

pub(crate) struct PollConfig {
    pub(crate) generation: u64,
    pub(crate) base_url: String,
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) auth: std::sync::Arc<std::sync::Mutex<crate::sunsynk::AuthState>>,
    pub(crate) serial: String,
    pub(crate) plant_id: Option<i64>,
    pub(crate) interval_seconds: u64,
    pub(crate) history_days: u64,
    pub(crate) database: std::sync::Arc<crate::storage::database::Database>,
    pub(crate) connection_log: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<String>>>,
    pub(crate) connection_log_revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub(crate) connection_log_signal: std::sync::Arc<tokio::sync::Notify>,
}
