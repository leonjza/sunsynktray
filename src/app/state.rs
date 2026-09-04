use crate::{
    domain::{EnergySnapshot, HistoryPoint, HistorySeries},
    storage::config::Settings,
};
use gpui_kit::Global;
use std::sync::{Arc, Mutex};

pub(crate) struct MonitorState {
    pub(crate) settings: Settings,
    data: Arc<Mutex<MonitorData>>,
}

struct MonitorData {
    snapshot: EnergySnapshot,
    live_data: bool,
    history: Arc<Vec<HistorySeries>>,
}

pub(crate) struct MonitorStateGlobal(pub Arc<MonitorState>);
impl Global for MonitorStateGlobal {}

impl MonitorState {
    pub(crate) fn new(settings: Settings) -> Arc<Self> {
        Arc::new(Self {
            settings,
            data: Arc::new(Mutex::new(MonitorData {
                snapshot: EnergySnapshot {
                    inverter_sn: "DEMO-SN-2026".into(),
                    pv_watts: 3240.0,
                    load_watts: 1180.0,
                    grid_watts: -2060.0,
                    battery_watts: 860.0,
                    battery_soc: 78.0,
                    updated_at: Some("Sample data".into()),
                    solar_yield_kwh: Some(18.4),
                    pv_to: Some(true),
                    to_load: Some(true),
                    to_grid: Some(true),
                    to_battery: Some(false),
                    battery_to: Some(true),
                    grid_to: Some(false),
                },
                live_data: false,
                history: Arc::new(vec![HistorySeries {
                    label: "pac".into(),
                    points: (0..24)
                        .map(|hour| HistoryPoint {
                            time: format!("{hour:02}:00"),
                            watts: 900.0 + (hour as f64 * 130.0).sin() * 600.0,
                        })
                        .collect(),
                }]),
            })),
        })
    }

    pub(crate) fn set_snapshot(&self, snapshot: EnergySnapshot) {
        let mut data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        data.snapshot = snapshot;
        data.live_data = true;
    }

    pub(crate) fn set_history(&self, history: Vec<HistorySeries>) {
        self.data
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .history = Arc::new(history);
    }

    pub(crate) fn has_live_data(&self) -> bool {
        self.data
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .live_data
    }

    pub(crate) fn snapshot(&self) -> EnergySnapshot {
        self.data
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .snapshot
            .clone()
    }

    pub(crate) fn history(&self) -> Arc<Vec<HistorySeries>> {
        self.data
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .history
            .clone()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Dashboard,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrayMetric {
    Soc,
    Load,
    Solar,
}

impl TrayMetric {
    pub(crate) fn from_saved(value: Option<&str>) -> Option<Self> {
        match value {
            Some("soc") => Some(Self::Soc),
            Some("load") => Some(Self::Load),
            Some("solar") => Some(Self::Solar),
            _ => None,
        }
    }

    pub(crate) const fn saved_name(self) -> &'static str {
        match self {
            Self::Soc => "soc",
            Self::Load => "load",
            Self::Solar => "solar",
        }
    }

    pub(crate) fn value(self, snapshot: &EnergySnapshot) -> String {
        match self {
            Self::Soc => format!("{:.0}%", snapshot.battery_soc),
            Self::Load => tray_power(snapshot.load_watts),
            Self::Solar => tray_power(snapshot.pv_watts),
        }
    }
}

pub(crate) enum ConnectionState {
    Unconfigured,
    Connecting,
    Connected,
    Stale,
    Error(String),
}

fn tray_power(watts: f64) -> String {
    #[cfg(target_os = "windows")]
    if watts.abs() >= 1000. {
        format!("{:.1} kW", watts / 1000.)
    } else {
        format!("{watts:.0} W")
    }

    #[cfg(not(target_os = "windows"))]
    if watts.abs() >= 1000. {
        format!("{:.1}", watts / 1000.)
    } else {
        format!("{watts:.0}")
    }
}
