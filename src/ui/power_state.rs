use crate::domain::EnergySnapshot;
use gpui_kit::component::{StyledExt, Theme};
use gpui_kit::{div, rgb, Hsla, IntoElement, ParentElement, Styled};

pub(crate) const FLOW_TOLERANCE_WATTS: f64 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowerStateKind {
    Offline,
    Solar,
    SolarCharging,
    SolarBattery,
    Battery,
    Grid,
    GridCharging,
    Exporting,
    Idle,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PowerStateTone {
    Green,
    Yellow,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PowerState {
    pub(crate) kind: PowerStateKind,
    pub(crate) title: &'static str,
    pub(crate) detail: &'static str,
    pub(crate) tone: PowerStateTone,
}

impl PowerState {
    const fn new(
        kind: PowerStateKind,
        title: &'static str,
        detail: &'static str,
        tone: PowerStateTone,
    ) -> Self {
        Self {
            kind,
            title,
            detail,
            tone,
        }
    }
}

pub(crate) fn classify(snapshot: &EnergySnapshot, live: bool) -> PowerState {
    if !live {
        return PowerState::new(
            PowerStateKind::Offline,
            "Offline",
            "Showing cached data",
            PowerStateTone::Neutral,
        );
    }

    let pv_active = snapshot.pv_watts > FLOW_TOLERANCE_WATTS;
    let battery_active = active(snapshot.battery_watts);
    let grid_active = active(snapshot.grid_watts);
    let battery_discharging = battery_active && battery_to_inverter(snapshot);
    let battery_charging = battery_active && !battery_discharging;
    let grid_exporting = grid_active && inverter_to_grid(snapshot);
    let grid_importing = grid_active && !grid_exporting;
    let solar_to_load = pv_active && snapshot.to_load.unwrap_or(true);

    if pv_active && grid_exporting && !battery_discharging {
        return PowerState::new(
            PowerStateKind::Exporting,
            "Sending surplus to grid",
            "Solar is producing more than the home needs",
            PowerStateTone::Green,
        );
    }
    if pv_active && battery_charging {
        return PowerState::new(
            PowerStateKind::SolarCharging,
            "Charging from solar",
            "Solar is supplying the home and battery",
            PowerStateTone::Green,
        );
    }
    if pv_active && battery_discharging {
        return PowerState::new(
            PowerStateKind::SolarBattery,
            "Solar + battery support",
            "Solar and battery are supplying the home",
            PowerStateTone::Green,
        );
    }
    if pv_active && solar_to_load {
        return PowerState::new(
            PowerStateKind::Solar,
            "Running on solar",
            "Solar is supplying the home",
            PowerStateTone::Green,
        );
    }
    if !pv_active && battery_discharging {
        return PowerState::new(
            PowerStateKind::Battery,
            "Running on battery",
            "Battery is supplying the home",
            PowerStateTone::Yellow,
        );
    }
    if !pv_active && battery_charging && grid_importing {
        return PowerState::new(
            PowerStateKind::GridCharging,
            "Charging from the grid",
            "The grid is supplying the home and battery",
            PowerStateTone::Yellow,
        );
    }
    if !pv_active && grid_importing {
        return PowerState::new(
            PowerStateKind::Grid,
            "Using grid power",
            "The grid is supplying the home",
            PowerStateTone::Yellow,
        );
    }
    if !pv_active && !battery_active && !grid_active && !active(snapshot.load_watts) {
        return PowerState::new(
            PowerStateKind::Idle,
            "No active power flow",
            "The system is idle",
            PowerStateTone::Neutral,
        );
    }

    PowerState::new(
        PowerStateKind::Unknown,
        "Monitoring power flow",
        "Power-flow readings are changing",
        PowerStateTone::Neutral,
    )
}

pub(crate) fn render(theme: &Theme, snapshot: &EnergySnapshot, live: bool) -> impl IntoElement {
    let state = classify(snapshot, live);
    let color = match state.tone {
        PowerStateTone::Green => rgb(0x34c759).into(),
        PowerStateTone::Yellow => rgb(0xf5b942).into(),
        PowerStateTone::Neutral => theme.muted_foreground,
    };
    state_card(theme, state, color)
}

fn state_card(theme: &Theme, state: PowerState, color: Hsla) -> impl IntoElement {
    div()
        .h_flex()
        .items_center()
        .gap_3()
        .p_3()
        .bg(theme.muted)
        .rounded(theme.radius)
        .border_1()
        .border_color(theme.border)
        .child(div().size_3().rounded_full().bg(color))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(div().text_sm().child(state.title))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!("· {}", state.detail)),
                ),
        )
}

fn active(watts: f64) -> bool {
    watts.abs() > FLOW_TOLERANCE_WATTS
}

fn battery_to_inverter(snapshot: &EnergySnapshot) -> bool {
    snapshot
        .battery_to
        .or(snapshot.to_battery.map(|to_battery| !to_battery))
        .unwrap_or(snapshot.battery_watts < -FLOW_TOLERANCE_WATTS)
}

fn inverter_to_grid(snapshot: &EnergySnapshot) -> bool {
    snapshot
        .grid_to
        .or(snapshot.to_grid)
        .unwrap_or(snapshot.grid_watts < -FLOW_TOLERANCE_WATTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(pv: f64, battery: f64, grid: f64, load: f64) -> EnergySnapshot {
        EnergySnapshot {
            pv_watts: pv,
            battery_watts: battery,
            grid_watts: grid,
            load_watts: load,
            battery_to: Some(battery < -FLOW_TOLERANCE_WATTS),
            to_load: Some(true),
            to_grid: Some(grid < -FLOW_TOLERANCE_WATTS),
            ..Default::default()
        }
    }

    #[test]
    fn classifies_the_main_power_states() {
        assert_eq!(
            classify(&snapshot(0., -500., 0., 500.), true).kind,
            PowerStateKind::Battery
        );
        assert_eq!(
            classify(&snapshot(1000., 0., 0., 500.), true).kind,
            PowerStateKind::Solar
        );
        assert_eq!(
            classify(&snapshot(1000., 300., 0., 500.), true).kind,
            PowerStateKind::SolarCharging
        );
        assert_eq!(
            classify(&snapshot(0., 300., 500., 500.), true).kind,
            PowerStateKind::GridCharging
        );
        assert_eq!(
            classify(&snapshot(0., 0., 500., 500.), true).kind,
            PowerStateKind::Grid
        );
        assert_eq!(
            classify(&snapshot(1000., 0., -500., 500.), true).kind,
            PowerStateKind::Exporting
        );
    }

    #[test]
    fn treats_readings_within_twenty_watts_as_idle() {
        let state = classify(&snapshot(20., 20., -20., 20.), true);
        assert_eq!(state.kind, PowerStateKind::Idle);
    }

    #[test]
    fn reports_cached_data_as_offline() {
        assert_eq!(
            classify(&snapshot(1000., 0., 0., 500.), false).kind,
            PowerStateKind::Offline
        );
    }
}
