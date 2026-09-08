use crate::{
    domain::{EnergySnapshot, HistoryPoint, HistorySeries},
    storage::config::Settings,
};
use gpui_kit::Global;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

pub(crate) type HistoryPointIndex = HashMap<usize, HashMap<String, f64>>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistorySource {
    Cached,
    Network,
}

#[derive(Clone)]
pub(crate) struct HistorySnapshot {
    pub(crate) series: Arc<Vec<HistorySeries>>,
    pub(crate) index: Arc<HistoryPointIndex>,
    pub(crate) times: Arc<Vec<String>>,
    pub(crate) time_indices: Arc<HashMap<String, usize>>,
    pub(crate) power_indices: Arc<Vec<usize>>,
    pub(crate) soc_indices: Arc<Vec<usize>>,
    pub(crate) power_bounds: (f64, f64),
}

#[derive(Clone)]
pub(crate) struct MonitorDataSnapshot {
    pub(crate) snapshot: EnergySnapshot,
    pub(crate) history: HistorySnapshot,
}

pub(crate) struct MonitorState {
    pub(crate) settings: Settings,
    pub(crate) database: Arc<crate::storage::database::Database>,
    data: Arc<Mutex<MonitorData>>,
}

struct MonitorData {
    snapshot: EnergySnapshot,
    live_data: bool,
    history: HistorySnapshot,
}

pub(crate) struct MonitorStateGlobal(pub Arc<MonitorState>);
impl Global for MonitorStateGlobal {}

impl MonitorState {
    pub(crate) fn new(
        settings: Settings,
        database: Arc<crate::storage::database::Database>,
    ) -> Arc<Self> {
        let history = Arc::new(vec![HistorySeries {
            label: "pac".into(),
            points: (0..24)
                .map(|hour| HistoryPoint {
                    time: format!("{hour:02}:00"),
                    watts: 900.0 + (hour as f64 * 130.0).sin() * 600.0,
                })
                .collect(),
        }]);
        let history = make_history_snapshot(history);
        Arc::new(Self {
            settings,
            database,
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
                history,
            })),
        })
    }

    pub(crate) fn set_snapshot(&self, snapshot: EnergySnapshot) {
        let mut data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        data.snapshot = snapshot;
        data.live_data = true;
    }

    pub(crate) fn set_cached_data(&self, snapshot: EnergySnapshot, history: Vec<HistorySeries>) {
        let mut data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        data.snapshot = snapshot;
        data.live_data = true;
        if !history.is_empty() {
            data.history = make_history_snapshot(Arc::new(history));
        }
    }

    pub(crate) fn clear_cached_data(&self) {
        let mut data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        data.snapshot = EnergySnapshot::default();
        data.live_data = false;
        data.history = make_history_snapshot(Arc::new(Vec::new()));
    }

    pub(crate) fn set_history(&self, history: Vec<HistorySeries>) {
        let history = make_history_snapshot(Arc::new(history));
        let mut data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        data.history = history;
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

    pub(crate) fn data_snapshot(&self) -> MonitorDataSnapshot {
        let data = self.data.lock().unwrap_or_else(|error| error.into_inner());
        MonitorDataSnapshot {
            snapshot: data.snapshot.clone(),
            history: data.history.clone(),
        }
    }
}

fn index_history(history: &[HistorySeries]) -> HistoryPointIndex {
    let mut index = HistoryPointIndex::new();
    for (series_index, series) in history.iter().enumerate() {
        let series_index = index.entry(series_index).or_default();
        for point in &series.points {
            series_index
                .entry(point.time.clone())
                .or_insert(point.watts);
        }
    }
    index
}

fn make_history_snapshot(history: Arc<Vec<HistorySeries>>) -> HistorySnapshot {
    let power_indices = power_indices(&history);
    let times = history_times(&history);
    let time_indices = times
        .iter()
        .enumerate()
        .map(|(index, time)| (time.clone(), index))
        .collect();
    HistorySnapshot {
        index: Arc::new(index_history(&history)),
        times: Arc::new(times),
        time_indices: Arc::new(time_indices),
        soc_indices: Arc::new(soc_indices(&history)),
        power_bounds: power_bounds(&history, &power_indices),
        power_indices: Arc::new(power_indices),
        series: history,
    }
}

fn history_times(history: &[HistorySeries]) -> Vec<String> {
    let mut times = Vec::new();
    let mut seen = HashSet::new();
    for point in history.iter().flat_map(|series| &series.points) {
        if seen.insert(point.time.clone()) {
            times.push(point.time.clone());
        }
    }
    times.sort_unstable();
    times
}

fn power_indices(history: &[HistorySeries]) -> Vec<usize> {
    history
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            (!series.label.to_ascii_lowercase().contains("soc")).then_some(index)
        })
        .collect()
}

fn soc_indices(history: &[HistorySeries]) -> Vec<usize> {
    history
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            series
                .label
                .to_ascii_lowercase()
                .contains("soc")
                .then_some(index)
        })
        .collect()
}

fn power_bounds(history: &[HistorySeries], indices: &[usize]) -> (f64, f64) {
    indices
        .iter()
        .filter_map(|&index| history.get(index))
        .flat_map(|series| series.points.iter().map(|point| point.watts))
        .chain(Some(0.))
        .fold((0.0_f64, 0.0_f64), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

#[cfg(test)]
mod tests {
    use super::index_history;
    use crate::domain::{HistoryPoint, HistorySeries};

    #[test]
    fn history_index_preserves_series_and_first_duplicate_value() {
        let history = vec![
            HistorySeries {
                label: "solar".into(),
                points: vec![
                    HistoryPoint {
                        time: "12:00".into(),
                        watts: 100.0,
                    },
                    HistoryPoint {
                        time: "12:00".into(),
                        watts: 999.0,
                    },
                ],
            },
            HistorySeries {
                label: "load".into(),
                points: vec![HistoryPoint {
                    time: "12:00".into(),
                    watts: 50.0,
                }],
            },
        ];

        let index = index_history(&history);
        assert_eq!(
            index.get(&0).and_then(|points| points.get("12:00")),
            Some(&100.0)
        );
        assert_eq!(
            index.get(&1).and_then(|points| points.get("12:00")),
            Some(&50.0)
        );
        assert!(!index.contains_key(&2));
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

#[derive(Clone)]
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
