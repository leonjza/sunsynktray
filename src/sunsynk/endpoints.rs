use super::{parsing::*, SunsynkClient};
use crate::domain::{EnergySnapshot, HistorySeries, InverterSummary};
use anyhow::{anyhow, bail, Result};
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

    pub(crate) async fn refresh_plant(
        &mut self,
        plant_id: i64,
        serial: &str,
    ) -> Result<(EnergySnapshot, Option<Vec<HistorySeries>>)> {
        let today = chrono::Local::now().date_naive().to_string();
        let realtime = self
            .get(
                &format!("/api/v1/plant/{plant_id}/realtime"),
                Some(&[("id", plant_id.to_string())]),
            )
            .await;
        // Flow is the authoritative live-state response. Keep the live
        // dashboard usable if an auxiliary energy endpoint is unavailable.
        let day = self
            .get(
                &format!("/api/v1/plant/energy/{plant_id}/day"),
                Some(&[
                    ("lan", "en".to_owned()),
                    ("date", today.clone()),
                    ("id", plant_id.to_string()),
                ]),
            )
            .await;
        let (realtime, day) = match (realtime, day) {
            (Ok(realtime), Ok(day)) => (Some(realtime), Some(day)),
            (realtime, day) => {
                if let Err(ref error) = realtime {
                    tracing::warn!(%error, "SunSynk realtime data unavailable; using flow data");
                }
                if let Err(ref error) = day {
                    tracing::warn!(%error, "SunSynk daily energy data unavailable; using flow data");
                }
                (realtime.ok(), day.ok())
            }
        };
        // The flow endpoint is both the live power-flow source and, for the
        // current day, the chart source. Never use a historical chart date for
        // live dashboard values.
        let flow = self
            .get(
                &format!("/api/v1/plant/energy/{plant_id}/flow"),
                Some(&[("date", today.clone())]),
            )
            .await?;
        let live = flow_object(&flow)
            .ok_or_else(|| anyhow!("SunSynk plant flow response contained no live readings"))?;
        let mut snapshot = snapshot_from_flow(live, serial);
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
        let history = {
            let history = history_series(&flow);
            if history.is_empty() {
                day.as_ref().map(history_series).unwrap_or_default()
            } else {
                history
            }
        };
        if snapshot.solar_yield_kwh.is_none() {
            let day_history = day.as_ref().map(history_series).unwrap_or_default();
            snapshot.solar_yield_kwh = snapshot
                .solar_yield_kwh
                .or_else(|| daily_solar_yield_from_history(&day_history));
        }
        Ok((snapshot, (!history.is_empty()).then_some(history)))
    }

    pub async fn inspect_endpoint(
        &mut self,
        path: &str,
        params: Option<&[(&str, String)]>,
    ) -> Result<Value> {
        self.get(path, params).await
    }

    pub async fn history(&mut self, plant_id: i64, date: &str) -> Result<Vec<HistorySeries>> {
        let params = [
            ("lan", "en".to_owned()),
            ("date", date.to_owned()),
            ("id", plant_id.to_string()),
        ];
        // Use the common authenticated GET path so history gets the same
        // expiry retry and refresh-token fallback as live readings.
        let day = self
            .get(
                &format!("/api/v1/plant/energy/{plant_id}/day"),
                Some(&params),
            )
            .await?;
        let flow = self
            .get(
                &format!("/api/v1/plant/energy/{plant_id}/flow"),
                Some(&[("date", date.to_owned())]),
            )
            .await?;
        let from_flow = history_series(&flow);
        Ok(if from_flow.is_empty() {
            history_series(&day)
        } else {
            from_flow
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::parsing::{flow_object, history_series, snapshot_from_flow};
    use serde_json::Value;

    #[test]
    fn captured_api_fixture_contains_parseable_dashboard_data() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../fixtures/api/latest.json")).unwrap();
        let responses = fixture["responses"].as_object().unwrap();
        let flow = flow_object(&responses["flow"]).unwrap();
        let snapshot = snapshot_from_flow(flow, "2105287329");
        assert_eq!(snapshot.pv_watts, 463.0);
        assert_eq!(snapshot.load_watts, 438.0);
        assert_eq!(snapshot.battery_soc, 99.0);
        assert!(!history_series(&responses["plant_energy_day"]).is_empty());
        assert!(flow_object(&responses["flow"]).is_some());
    }
}
