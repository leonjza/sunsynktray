use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
    /// Live grid availability from the inverter grid realtime endpoint.
    /// This is intentionally omitted from persisted snapshots because it is
    /// refreshed independently and should not be presented as current later.
    #[serde(skip)]
    pub(crate) grid_connected: Option<bool>,
    #[serde(skip)]
    pub(crate) grid_voltage: Option<f64>,
    #[serde(skip)]
    pub(crate) grid_frequency: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct HistoryPoint {
    pub(crate) time: String,
    pub(crate) watts: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NotificationMessage {
    #[serde(default, deserialize_with = "deserialize_string")]
    pub(crate) id: String,
    #[serde(default, deserialize_with = "deserialize_string")]
    pub(crate) sn: String,
    #[serde(
        rename = "type",
        default,
        deserialize_with = "deserialize_optional_i64"
    )]
    pub(crate) notice_type: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub(crate) status: Option<i64>,
    #[serde(
        rename = "messageType",
        default,
        deserialize_with = "deserialize_optional_i64"
    )]
    pub(crate) message_type: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub(crate) time: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) create_at: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) update_at: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) station_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub(crate) soc: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) description: Option<String>,
    #[serde(
        rename = "noticeTittle",
        default,
        deserialize_with = "deserialize_optional_string"
    )]
    pub(crate) notice_title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) notice_description: Option<String>,
}

fn deserialize_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Value>::deserialize(deserializer)?
        .and_then(value_string)
        .unwrap_or_default())
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<Value>::deserialize(deserializer)?.and_then(value_string))
}

fn value_string(value: Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        Option::<Value>::deserialize(deserializer)?.and_then(|value| match value {
            Value::Number(value) => value
                .as_i64()
                .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok())),
            Value::String(value) => value.trim().parse().ok(),
            _ => None,
        }),
    )
}

fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        Option::<Value>::deserialize(deserializer)?.and_then(|value| {
            let number = match value {
                Value::Number(value) => value.as_f64(),
                Value::String(value) => value.trim().parse().ok(),
                _ => None,
            }?;
            number.is_finite().then_some(number)
        }),
    )
}
