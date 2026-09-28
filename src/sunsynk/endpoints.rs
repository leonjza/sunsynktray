use super::{
    client::response_indicates_expired, error::AuthenticationExpired, parsing::*, SunsynkClient,
};
use crate::domain::{EnergySnapshot, HistorySeries, InverterSummary, NotificationMessage};
use anyhow::{anyhow, bail, Context, Result};
use futures_util::future::join4;
use serde_json::Value;

impl SunsynkClient {
    pub(crate) async fn list_inverters(&mut self) -> Result<Vec<InverterSummary>> {
        self.report_progress("Discovering plants and inverters…");
        let mut page = 1;
        let mut result = Vec::new();
        loop {
            if page > 20 {
                bail!("SunSynk returned too many inverter pages");
            }
            let params = [
                ("page", page.to_string()),
                ("limit", "50".into()),
                ("total", "0".into()),
                ("status", "-1".into()),
                ("sn", "".into()),
                ("plantId", "".into()),
                ("type", "-2".into()),
                ("softVer", "".into()),
                ("hmiVer", "".into()),
                ("agentCompanyId", "-1".into()),
                ("gsn", "".into()),
            ];
            let value = self.get("/api/v1/inverters", Some(&params)).await?;
            let infos = data(&value)?
                .get("infos")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let count = infos.len();
            result.extend(infos.into_iter().map(|item| {
                InverterSummary {
                    serial: string(&item, "sn"),
                    plant_id: item
                        .get("plant")
                        .and_then(|plant| plant.get("id"))
                        .and_then(integer),
                    alias: string(&item, "alias"),
                    model: string(&item, "model"),
                    plant_name: item
                        .get("plant")
                        .map(|p| string(p, "name"))
                        .unwrap_or_default(),
                    status: item
                        .get("status")
                        .and_then(Value::as_i64)
                        .unwrap_or_default(),
                }
            }));
            if count < 50 {
                break;
            }
            page += 1;
        }
        Ok(result)
    }

    pub(crate) async fn notification_count(&self) -> Result<u64> {
        let response = self.get("/api/v1/messageTd/count", None).await?;
        response
            .get("data")
            .and_then(Value::as_u64)
            .context("SunSynk notification count response had no numeric data")
    }

    pub(crate) async fn notifications(
        &self,
        page_size: u64,
        page_number: u64,
        language: &str,
    ) -> Result<Vec<NotificationMessage>> {
        let path = "/api/v1/messageTd/messages";
        let params = [
            ("pageSize", page_size.to_string()),
            ("pageNumber", page_number.to_string()),
            ("status", String::new()),
            ("lan", language.to_owned()),
        ];
        let response = self.get(path, Some(&params)).await?;
        let records = response
            .get("data")
            .and_then(|data| data.get("infos"))
            .and_then(|infos| infos.get("records"))
            .and_then(Value::as_array)
            .context("SunSynk notification response did not contain data.infos.records")?;
        records
            .iter()
            .cloned()
            .map(serde_json::from_value)
            .collect::<Result<Vec<_>, _>>()
            .context("could not parse SunSynk notification records")
    }

    pub(crate) async fn mark_notification_read(&self, id: &str, notice_type: i64) -> Result<()> {
        let params = [("id", id.to_owned()), ("type", notice_type.to_string())];
        self.post("/api/v1/messageTd/readMessage", Some(&params), None)
            .await?;
        Ok(())
    }

    pub(crate) async fn refresh_plant(
        &self,
        plant_id: i64,
        serial: &str,
        include_today_history: bool,
    ) -> Result<(EnergySnapshot, Option<Vec<HistorySeries>>)> {
        let today = chrono::Local::now().date_naive().to_string();
        // Authenticate once, then fetch the independent readings in
        // parallel. Each clone shares reqwest's connection pool while keeping
        // the client's mutable token state isolated from the other requests.
        self.ensure_authenticated().await?;
        let realtime_path = format!("/api/v1/plant/{plant_id}/realtime");
        let flow_path = format!("/api/v1/plant/energy/{plant_id}/flow");
        let day_path = format!("/api/v1/plant/energy/{plant_id}/day");
        let grid_path = format!("/api/v1/inverter/grid/{serial}/realtime");
        let mut responses = self
            .parallel_readings(
                &realtime_path,
                &flow_path,
                &day_path,
                &grid_path,
                serial,
                plant_id,
                &today,
                include_today_history,
            )
            .await?;
        if responses
            .iter()
            .any(|response| response.as_ref().err().is_some_and(is_auth_expired))
        {
            self.auth
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .access_token = None;
            self.ensure_authenticated().await?;
            responses = self
                .parallel_readings(
                    &realtime_path,
                    &flow_path,
                    &day_path,
                    &grid_path,
                    serial,
                    plant_id,
                    &today,
                    include_today_history,
                )
                .await?;
        }
        let [realtime, flow, day, grid] = responses;
        // Flow is the authoritative live-state response. Keep the live
        // dashboard usable if an auxiliary energy endpoint is unavailable.
        let realtime = match realtime {
            Ok(realtime) => Some(realtime),
            Err(error) => {
                tracing::warn!(%error, "SunSynk realtime data unavailable; using flow data");
                None
            }
        };
        // The flow endpoint is the authoritative live power-flow source.
        let flow = flow?;
        let live = flow_object(&flow)
            .ok_or_else(|| anyhow!("SunSynk plant flow response contained no live readings"))?;
        let mut snapshot = snapshot_from_flow(live, serial);
        if let Some(grid) = grid.as_ref().ok() {
            snapshot.grid_voltage = grid_voltage(grid);
            snapshot.grid_frequency = grid_frequency(grid);
            snapshot.grid_connected = match (snapshot.grid_voltage, snapshot.grid_frequency) {
                (Some(voltage), Some(frequency)) => {
                    Some(voltage >= 100.0 && (45.0..=65.0).contains(&frequency))
                }
                _ => None,
            };
        }
        snapshot.solar_yield_kwh = realtime
            .as_ref()
            .and_then(|value| data(value).ok())
            .and_then(|summary| first_number(summary, &["etoday", "pvToday", "solarYield"]))
            .or(snapshot.solar_yield_kwh);
        snapshot.updated_at = realtime
            .as_ref()
            .and_then(|value| data(value).ok())
            .and_then(|summary| first_string(summary, &["updateAt", "updatedAt", "updateTime"]))
            .or(snapshot.updated_at);
        let history = match day {
            Ok(day) => history_series(&day),
            Err(error) => {
                tracing::warn!(%error, "SunSynk day history unavailable");
                Vec::new()
            }
        };
        Ok((snapshot, (!history.is_empty()).then_some(history)))
    }

    async fn parallel_readings(
        &self,
        realtime_path: &str,
        flow_path: &str,
        day_path: &str,
        grid_path: &str,
        serial: &str,
        plant_id: i64,
        today: &str,
        include_today_history: bool,
    ) -> Result<[Result<Value>; 4]> {
        let realtime_client = self.clone();
        let flow_client = self.clone();
        let day_client = self.clone();
        let grid_client = self.clone();
        let realtime_path = realtime_path.to_owned();
        let flow_path = flow_path.to_owned();
        let day_path = day_path.to_owned();
        let today = today.to_owned();
        let serial = serial.to_owned();
        let realtime = async move {
            realtime_client
                .get_authenticated(&realtime_path, Some(&[("id", plant_id.to_string())]))
                .await
        };
        let day_today = today.clone();
        let flow = async move {
            flow_client
                .get_authenticated(&flow_path, Some(&[("date", today)]))
                .await
        };
        let day = async move {
            if include_today_history {
                day_client
                    .get_authenticated(
                        &day_path,
                        Some(&[
                            ("lan", "en".into()),
                            ("date", day_today),
                            ("id", plant_id.to_string()),
                        ]),
                    )
                    .await
            } else {
                Ok(Value::Null)
            }
        };
        let grid = async move {
            tokio::time::timeout(
                std::time::Duration::from_secs(5),
                grid_client.get_authenticated(&grid_path, Some(&[("sn", serial)])),
            )
            .await
            .map_err(|_| anyhow!("SunSynk grid realtime request timed out"))?
        };
        let (realtime, flow, day, grid) = join4(realtime, flow, day, grid).await;
        Ok([realtime, flow, day, grid])
    }

    pub async fn inspect_endpoint(
        &self,
        path: &str,
        params: Option<&[(&str, String)]>,
    ) -> Result<(u16, Value)> {
        // Inspection output belongs in the inspector, not the connection log.
        let mut client = self.clone();
        client.request_log = None;
        client.ensure_authenticated().await?;
        match client.request_with_status("GET", path, params, None).await {
            Ok((_, response)) if response_indicates_expired(&response) => {
                client
                    .auth
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .access_token = None;
                client.ensure_authenticated().await?;
                client.request_with_status("GET", path, params, None).await
            }
            Err(error) if is_auth_expired(&error) => {
                client
                    .auth
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .access_token = None;
                client.ensure_authenticated().await?;
                client.request_with_status("GET", path, params, None).await
            }
            result => result,
        }
    }

    pub async fn history(&self, plant_id: i64, date: &str) -> Result<Vec<HistorySeries>> {
        let day_path = format!("/api/v1/plant/energy/{plant_id}/day");
        let params = [
            ("lan", "en".into()),
            ("date", date.to_owned()),
            ("id", plant_id.to_string()),
        ];
        // Route history through the same authenticated GET factory as every
        // other non-parallel request, including its expiry retry.
        Ok(history_series(&self.get(&day_path, Some(&params)).await?))
    }
}

fn grid_voltage(response: &Value) -> Option<f64> {
    let data = response.get("data")?;
    data.get("vip")?
        .as_array()?
        .iter()
        .filter_map(|channel| {
            channel
                .get("volt")
                .and_then(crate::sunsynk::parsing::parse_number)
        })
        .reduce(f64::max)
}

fn grid_frequency(response: &Value) -> Option<f64> {
    response
        .get("data")?
        .get("fac")
        .and_then(crate::sunsynk::parsing::parse_number)
}

fn is_auth_expired(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.downcast_ref::<AuthenticationExpired>().is_some())
}

#[cfg(test)]
mod tests {
    use super::super::parsing::{flow_object, history_series, snapshot_from_flow};
    use super::is_auth_expired;
    use crate::sunsynk::error::AuthenticationExpired;
    use anyhow::anyhow;
    use serde_json::json;

    #[test]
    fn sample_api_response_contains_parseable_dashboard_data() {
        let flow_response = json!({
            "data": {
                "pvPower": 463,
                "homeLoadPower": 438,
                "battPower": 25,
                "soc": 99,
                "pv": [{"power": 240}, {"power": 223}]
            }
        });
        let day_response = json!({
            "data": {"infos": [{
                "label": "pv",
                "records": [{"time": "2026-09-06 12:00:00", "value": 463}]
            }]}
        });
        let flow = flow_object(&flow_response).unwrap();
        let snapshot = snapshot_from_flow(flow, "2105287329");
        assert_eq!(snapshot.pv_watts, 463.0);
        assert_eq!(snapshot.load_watts, 438.0);
        assert_eq!(snapshot.battery_soc, 99.0);
        assert!(!history_series(&day_response).is_empty());
        assert!(flow_object(&flow_response).is_some());
    }

    #[test]
    fn detects_auth_expiry_through_request_context() {
        let error = anyhow!(AuthenticationExpired).context("GET /api/v1/test");
        assert!(is_auth_expired(&error));
    }
}
