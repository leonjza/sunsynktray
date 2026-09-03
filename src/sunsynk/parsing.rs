use crate::domain::{EnergySnapshot, HistoryPoint, HistorySeries};
use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

pub(super) fn data(value: &Value) -> Result<&Map<String, Value>> {
    if value.get("data").is_some() {
        value
            .get("data")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("SunSynk returned an invalid data object"))
    } else {
        value
            .as_object()
            .ok_or_else(|| anyhow!("SunSynk returned an invalid data object"))
    }
}

pub(super) fn flow_object(value: &Value) -> Option<&Map<String, Value>> {
    if let Some(items) = value.as_array() {
        return items.iter().find_map(flow_object);
    }
    let object = value.as_object()?;
    let score = [
        "pvPower",
        "battPower",
        "gridOrMeterPower",
        "loadOrEpsPower",
        "homeLoadPower",
        "soc",
    ]
    .iter()
    .filter(|key| object.contains_key(**key))
    .count();
    if score >= 2 {
        return Some(object);
    }
    object.values().find_map(flow_object)
}

pub(super) fn snapshot_from_flow(flow: &Map<String, Value>, serial: &str) -> EnergySnapshot {
    EnergySnapshot {
        inverter_sn: serial.into(),
        pv_watts: flow_pv_watts(flow),
        load_watts: number(flow, "homeLoadPower").max(number(flow, "loadOrEpsPower")),
        grid_watts: number(flow, "gridOrMeterPower"),
        battery_watts: number(flow, "battPower"),
        battery_soc: number(flow, "soc"),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
        solar_yield_kwh: first_number(flow, &["etoday", "pvToday", "solarYield"]),
        pv_to: flow_pv_to(flow),
        to_load: flag(flow, "toLoad"),
        to_grid: flag(flow, "toGrid"),
        to_battery: flag(flow, "toBat"),
        battery_to: flag(flow, "batTo"),
        grid_to: flag(flow, "gridTo"),
    }
}

pub(super) fn history_series(value: &Value) -> Vec<HistorySeries> {
    let Some(object) = value.as_object() else {
        return value
            .as_array()
            .map(|items| items.iter().flat_map(history_series).collect())
            .unwrap_or_default();
    };
    if let Some(infos) = object.get("infos").and_then(Value::as_array) {
        let series = infos.iter().filter_map(history_item).collect::<Vec<_>>();
        if !series.is_empty() {
            return series;
        }
    }
    object.values().flat_map(history_series).collect()
}

pub(super) fn history_item(value: &Value) -> Option<HistorySeries> {
    let object = value.as_object()?;
    let label = string(value, "label");
    let points = object
        .get("records")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(|record| {
            Some(HistoryPoint {
                time: time_label(&string(record, "time")),
                watts: optional_number(record.as_object()?, "value")?,
            })
        })
        .collect::<Vec<_>>();
    (!label.is_empty() && !points.is_empty()).then_some(HistorySeries { label, points })
}

/// Plant day records are five-minute average power values. Some accounts do
/// not return a solar-energy counter from plant realtime, so derive the daily
/// solar yield from the same series the web chart displays.
pub(super) fn daily_solar_yield_from_history(history: &[HistorySeries]) -> Option<f64> {
    let series = history.iter().find(|series| {
        let label = series.label.to_ascii_lowercase();
        label == "pv" || label.contains("solar")
    })?;
    (!series.points.is_empty()).then(|| {
        let watts = series
            .points
            .iter()
            .map(|point| point.watts.max(0.0))
            .sum::<f64>();
        watts * (5.0 / 60.0) / 1000.0
    })
}

pub(super) fn first_number(value: &Map<String, Value>, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| optional_number(value, key))
}

pub(super) fn first_string(value: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    })
}

pub(super) fn string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .into()
}
pub(super) fn number(value: &Map<String, Value>, key: &str) -> f64 {
    value.get(key).and_then(parse_number).unwrap_or_default()
}

pub(super) fn flow_pv_watts(flow: &Map<String, Value>) -> f64 {
    let Some(channels) = flow.get("pv").and_then(Value::as_array) else {
        return number(flow, "pvPower");
    };

    let mut total = 0.0;
    let mut found_power = false;
    for channel in channels {
        if let Some(power) = channel.get("power").and_then(parse_number) {
            total += power;
            found_power = true;
        }
    }
    if found_power {
        total
    } else {
        number(flow, "pvPower")
    }
}

pub(super) fn flow_pv_to(flow: &Map<String, Value>) -> Option<bool> {
    let channels = flow.get("pv").and_then(Value::as_array)?;
    let directions = channels
        .iter()
        .filter_map(|channel| channel.get("toInv").and_then(Value::as_bool))
        .collect::<Vec<_>>();
    if directions.is_empty() {
        flag(flow, "pvTo")
    } else {
        Some(directions.into_iter().any(|to_inverter| to_inverter))
    }
}

pub(super) fn parse_number(value: &Value) -> Option<f64> {
    let number = value.as_f64().or_else(|| value.as_str()?.parse().ok())?;
    number.is_finite().then_some(number)
}

pub(super) fn optional_number(value: &Map<String, Value>, key: &str) -> Option<f64> {
    value.get(key).and_then(|v| {
        let number = v.as_f64().or_else(|| v.as_str()?.parse().ok())?;
        number.is_finite().then_some(number)
    })
}

pub(super) fn unsigned(value: &Value) -> Option<u64> {
    value.as_u64().or_else(|| value.as_str()?.parse().ok())
}

pub(super) fn integer(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| value.as_str()?.parse().ok())
}

pub(super) fn time_label(value: &str) -> String {
    let value = value.trim();
    let start = value
        .rfind('T')
        .map(|index| index + 1)
        .or_else(|| value.rfind(' ').map(|index| index + 1))
        .unwrap_or(0);
    value[start..].get(..5).unwrap_or(value).to_owned()
}

pub(super) fn flag(value: &Map<String, Value>, key: &str) -> Option<bool> {
    value.get(key).and_then(|value| {
        value
            .as_bool()
            .or_else(|| value.as_i64().map(|value| value != 0))
            .or_else(|| {
                value.as_str().and_then(|value| match value {
                    "true" | "1" => Some(true),
                    "false" | "0" => Some(false),
                    _ => None,
                })
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{HistoryPoint, HistorySeries};
    use serde_json::json;

    #[test]
    fn data_rejects_malformed_envelopes() {
        assert!(data(&json!({"data": null})).is_err());
        assert!(data(&json!({"data": []})).is_err());
        assert!(data(&json!(null)).is_err());
    }

    #[test]
    fn data_accepts_enveloped_and_raw_objects() {
        assert_eq!(data(&json!({"data": {"value": 1}})).unwrap()["value"], 1);
        assert_eq!(data(&json!({"value": 1})).unwrap()["value"], 1);
    }

    #[test]
    fn optional_numbers_preserve_zero_and_reject_non_finite_values() {
        let value = json!({"zero": 0, "text": "12.5", "bad": "not-a-number"});
        let object = value.as_object().unwrap();
        assert_eq!(optional_number(object, "zero"), Some(0.0));
        assert_eq!(optional_number(object, "text"), Some(12.5));
        assert_eq!(optional_number(object, "bad"), None);
    }

    #[test]
    fn flow_pv_uses_mppt_channel_total_when_aggregate_is_stale() {
        let value = json!({
            "pvPower": 0,
            "pv": [{"power": 240}, {"power": "223"}]
        });
        assert_eq!(flow_pv_watts(value.as_object().unwrap()), 463.0);
    }

    #[test]
    fn flow_pv_falls_back_to_aggregate_when_channels_are_missing() {
        let value = json!({"pvPower": "347"});
        assert_eq!(flow_pv_watts(value.as_object().unwrap()), 347.0);
    }

    #[test]
    fn flow_pv_direction_uses_mppt_channel_direction() {
        let value = json!({"pvTo": false, "pv": [{"toInv": true}, {"toInv": true}]});
        assert_eq!(flow_pv_to(value.as_object().unwrap()), Some(true));
    }

    #[test]
    fn expiry_accepts_numeric_strings() {
        assert_eq!(unsigned(&json!(3600)), Some(3600));
        assert_eq!(unsigned(&json!("3600")), Some(3600));
        assert_eq!(unsigned(&json!("invalid")), None);
    }

    #[test]
    fn plant_realtime_nested_readings_are_selected() {
        let value = json!({
            "data": {"realtime": {
                "pvPower": 366, "battPower": -27, "gridOrMeterPower": 20,
                "loadOrEpsPower": 352, "soc": 99, "etoday": 7.1
            }}
        });
        let flow = flow_object(&value).unwrap();
        let snapshot = snapshot_from_flow(flow, "serial");
        assert_eq!(snapshot.pv_watts, 366.0);
        assert_eq!(snapshot.solar_yield_kwh, Some(7.1));
    }

    #[test]
    fn plant_history_infos_are_parsed_without_an_extra_array_level() {
        let value = json!({
            "data": {"infos": [{"label": "PV", "records": [
                {"time": "08:00", "value": "123"}
            ]}]}
        });
        let history = history_series(&value);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].label, "PV");
        assert_eq!(history[0].points[0].watts, 123.0);
    }

    #[test]
    fn daily_solar_yield_can_be_derived_from_five_minute_day_records() {
        let history = vec![HistorySeries {
            label: "PV".into(),
            points: vec![
                HistoryPoint {
                    time: "00:00".into(),
                    watts: 600.0,
                },
                HistoryPoint {
                    time: "00:05".into(),
                    watts: 600.0,
                },
            ],
        }];
        assert_eq!(daily_solar_yield_from_history(&history), Some(0.1));
    }
}
