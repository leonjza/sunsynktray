use crate::domain::{EnergySnapshot, HistorySeries, InverterSummary};

pub(crate) enum PollResult {
    Connected {
        generation: u64,
        inverters: Vec<InverterSummary>,
        snapshot: Option<EnergySnapshot>,
        selected_serial: Option<String>,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
    },
    PollStarted,
    Progress {
        message: String,
    },
    History(Vec<HistorySeries>),
    HistoryFailure {
        date: chrono::NaiveDate,
        error: String,
    },
    Snapshot {
        snapshot: EnergySnapshot,
        refresh_token: Option<String>,
        history: Option<Vec<HistorySeries>>,
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
    Reconfigure {
        base_url: String,
        email: String,
        password: String,
        serial: String,
        plant_id: Option<i64>,
        refresh_token: Option<String>,
        interval: u64,
    },
    HistoryDate(chrono::NaiveDate),
}

pub(crate) struct PollConfig {
    pub(crate) base_url: String,
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) serial: String,
    pub(crate) plant_id: Option<i64>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) interval_seconds: u64,
}
