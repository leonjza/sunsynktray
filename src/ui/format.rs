use gpui::{rgb, Hsla};

pub(crate) fn history_value(label: &str, value: f64) -> String {
    if label.to_ascii_lowercase().contains("soc") {
        format!("{value:.0}%")
    } else {
        format_power(value)
    }
}

pub(crate) fn history_colors() -> [Hsla; 5] {
    [
        rgb(0xf5b942).into(),
        rgb(0xa78bfa).into(),
        rgb(0xfb7185).into(),
        rgb(0x2dd4bf).into(),
        rgb(0x4ade80).into(),
    ]
}

pub(crate) fn series_color_index(label: &str) -> usize {
    match label.to_ascii_lowercase().as_str() {
        "ppv" | "p-pv" | "pv" => 0,
        "battpower" | "p-bat" | "battery" => 1,
        "loadorepspower" | "p-load" | "load" => 2,
        "gridormeterpower" | "p-grid" | "grid" => 3,
        "soc" => 4,
        _ => 0,
    }
}

pub(crate) fn history_label(label: &str) -> String {
    match label.to_ascii_lowercase().as_str() {
        "pac" => "Output".into(),
        "ppv" | "p-pv" | "pv" => "Solar".into(),
        "soc" => "SOC".into(),
        "loadorepspower" | "p-load" | "load" => "Load".into(),
        "gridormeterpower" | "p-grid" | "grid" => "Grid".into(),
        "battpower" | "p-bat" | "battery" => "Battery".into(),
        _ => label.into(),
    }
}

pub(crate) fn format_power(watts: f64) -> String {
    if watts >= 1000. {
        format!("{:.2} kW", watts / 1000.)
    } else {
        format!("{watts:.0} W")
    }
}

pub(crate) fn format_energy(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.1} kWh"))
        .unwrap_or_else(|| "—".into())
}
