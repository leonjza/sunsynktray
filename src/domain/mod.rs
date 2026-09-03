use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct EnergySnapshot {
    pub(crate) inverter_sn: String,
    pub(crate) pv_watts: f64,
    pub(crate) load_watts: f64,
    pub(crate) grid_watts: f64,
    pub(crate) battery_watts: f64,
    pub(crate) battery_soc: f64,
    pub(crate) updated_at: Option<String>,
    pub(crate) solar_yield_kwh: Option<f64>,
    pub(crate) pv_to: Option<bool>,
    pub(crate) to_load: Option<bool>,
    pub(crate) to_grid: Option<bool>,
    pub(crate) to_battery: Option<bool>,
    pub(crate) battery_to: Option<bool>,
    pub(crate) grid_to: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct HistoryPoint {
    pub(crate) time: String,
    pub(crate) watts: f64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct HistorySeries {
    pub(crate) label: String,
    pub(crate) points: Vec<HistoryPoint>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct InverterSummary {
    pub(crate) serial: String,
    pub(crate) plant_id: Option<i64>,
    pub(crate) alias: String,
    pub(crate) model: String,
    pub(crate) plant_name: String,
    pub(crate) status: i64,
}
