use gpui_kit::{actions, App, Global, MenuItem};
use gpui_tray::{Icon, Tray};

#[cfg(not(target_os = "macos"))]
use gpui_kit::BorrowAppContext;

#[cfg(target_os = "windows")]
use super::windows_tray_icon;

actions!(suntray_tray, [OpenDashboard, Quit]);

pub(crate) struct TrayState {
    pub(crate) tray: Tray,
    #[cfg(not(target_os = "macos"))]
    icon_key: Option<IconKey>,
}
impl Global for TrayState {}

#[cfg(not(target_os = "macos"))]
#[derive(Clone, Debug, Eq, PartialEq)]
struct IconKey {
    value: Option<String>,
    symbol: String,
}

pub(crate) fn install(cx: &mut App) {
    cx.on_action(open_dashboard);
    cx.on_action(quit);
    match Tray::builder()
        .icon(icon_for(None, cx, "bolt.fill"))
        .macos_system_symbol("bolt.fill")
        .tooltip("SunTray")
        .menu(|_| {
            vec![
                MenuItem::action("Open SunTray", OpenDashboard),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ]
        })
        .build(cx)
    {
        Ok(tray) => cx.set_global(TrayState {
            tray,
            #[cfg(not(target_os = "macos"))]
            icon_key: Some(IconKey {
                value: None,
                symbol: "bolt.fill".into(),
            }),
        }),
        Err(error) => tracing::error!("failed to create tray icon: {error}"),
    }
}

pub(crate) fn update(cx: &mut App, value: Option<&str>, symbol: &str, tooltip: &str) {
    let tray = cx.try_global::<TrayState>().map(|state| state.tray.clone());
    if let Some(tray) = tray {
        #[cfg(target_os = "macos")]
        {
            // AppKit renders status-item titles with the native menu-bar font.
            // Rasterizing the number into an NSImage makes it blurry and gives
            // it the wrong metrics, especially on Retina displays.
            let icon = value.is_none().then(|| icon_for(None, cx, symbol));
            if let Err(error) = tray.set_macos_system_symbol(Some(symbol), cx) {
                tracing::error!("failed to update macOS tray symbol: {error}");
            }
            if let Err(error) = tray.set_icon(icon, cx) {
                tracing::error!("failed to update tray icon: {error}");
            }
            if let Err(error) = tray.set_title(value.map(str::to_owned), cx) {
                tracing::error!("failed to update tray title: {error}");
            }
            if let Err(error) = tray.set_tooltip(Some(tooltip.to_owned()), cx) {
                tracing::error!("failed to update tray tooltip: {error}");
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let icon_key = IconKey {
                value: value.map(compact_value),
                symbol: symbol.to_owned(),
            };
            let icon_changed = cx
                .try_global::<TrayState>()
                .map(|state| state.icon_key.as_ref() != Some(&icon_key))
                .unwrap_or(true);
            if icon_changed {
                let icon = icon_for(value, cx, symbol);
                match tray.set_icon(Some(icon), cx) {
                    Ok(()) => cx.update_global::<TrayState, _>(|state, _| {
                        state.icon_key = Some(icon_key);
                    }),
                    Err(error) => tracing::error!("failed to update tray icon: {error}"),
                }
            }
            if let Err(error) = tray.set_tooltip(Some(tooltip.to_owned()), cx) {
                tracing::error!("failed to update tray tooltip: {error}");
            }
        }
    }
}

fn open_dashboard(_: &OpenDashboard, cx: &mut App) {
    cx.activate(true);
    let windows = cx.windows();
    if windows.is_empty() {
        if let Some(state) = cx
            .try_global::<crate::app::MonitorStateGlobal>()
            .map(|state| state.0.clone())
        {
            crate::app::open_main_window(cx, state);
        }
    }
    for window in cx.windows() {
        let _ = window.update(cx, |_, window, _| {
            #[cfg(target_os = "windows")]
            crate::platform::show_main_window(window);
            window.activate_window();
        });
    }
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

#[cfg(not(target_os = "windows"))]
fn icon_for(value: Option<&str>, cx: &App, symbol: &str) -> Icon {
    #[cfg(target_os = "macos")]
    const FONT_FAMILY: &str = ".SF Compact";
    #[cfg(target_os = "windows")]
    const FONT_FAMILY: &str = "Segoe UI Semibold";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    const FONT_FAMILY: &str = "Noto Sans";

    #[cfg(target_os = "windows")]
    let (width, height, base_font_size, text_x, text_y) = windows_tray_layout();
    #[cfg(not(target_os = "windows"))]
    let (width, height, base_font_size, text_x, text_y) = (24, 18, 11.0, 12, 9.0);

    let color = metric_color(value, symbol);
    let text_value = value.map(compact_value);
    let font_size = text_value
        .as_ref()
        .map(|value| (base_font_size * 3.0 / value.chars().count().max(3) as f32).max(7.0))
        .unwrap_or(base_font_size);
    #[cfg(target_os = "windows")]
    const STROKE_WIDTH: f32 = 1.5;
    #[cfg(not(target_os = "windows"))]
    const STROKE_WIDTH: f32 = 1.15;
    let body = match value {
        Some(_) => format!(
            r##"<text x="{text_x}" y="{text_y}" text-anchor="middle" dominant-baseline="middle" font-family="{FONT_FAMILY}" font-size="{font_size}" font-weight="600" fill="none" stroke="#101010" stroke-width="{STROKE_WIDTH}" stroke-linejoin="round">{}</text><text x="{text_x}" y="{text_y}" text-anchor="middle" dominant-baseline="middle" font-family="{FONT_FAMILY}" font-size="{font_size}" font-weight="600" fill="{color}">{}</text>"##,
            escape_xml(text_value.as_deref().unwrap_or_default()),
            escape_xml(text_value.as_deref().unwrap_or_default())
        ),
        None => bolt_path(width).into(),
    };
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{body}</svg>"#
    );
    let image = gpui_kit::Image::from_bytes(gpui_kit::ImageFormat::Svg, svg.into_bytes());
    Icon::from_gpui(&image, cx).unwrap_or_else(|error| {
        tracing::warn!(%error, "could not render tray SVG");
        fallback_icon()
    })
}

#[cfg(target_os = "windows")]
fn icon_for(value: Option<&str>, _cx: &App, symbol: &str) -> Icon {
    windows_tray_icon::render(value, symbol).unwrap_or_else(|error| {
        tracing::warn!(%error, "could not render Windows tray icon");
        windows_tray_icon::fallback()
    })
}

fn fallback_icon() -> Icon {
    let size = 16;
    let mut rgba = vec![0; size * size * 4];
    for y in 4..28 {
        for x in 8..24 {
            if (x < 16 && y > x / 2 + 4) || (x >= 16 && y < 20 && y > 7) {
                let offset = (y * size + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&[255, 196, 0, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, size as u32, size as u32).expect("fallback tray icon is valid")
}

#[cfg(target_os = "windows")]
fn windows_tray_layout() -> (i32, i32, f32, i32, f32) {
    use windows::Win32::{
        Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX},
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON},
    };

    let reference = unsafe { GetDC(None) };
    if reference.is_invalid() {
        return (16, 16, 10.0, 8, 8.0);
    }
    let dpi = unsafe { GetDeviceCaps(Some(reference), LOGPIXELSX).max(96) } as f32 / 96.0;
    unsafe {
        ReleaseDC(None, reference);
    }
    let size = ((unsafe { GetSystemMetrics(SM_CXSMICON).max(16) } as f32) * dpi).round() as i32;
    (
        size,
        size,
        (size as f32 * 0.62).max(8.0),
        size / 2,
        size as f32 / 2.0,
    )
}

fn metric_color(value: Option<&str>, symbol: &str) -> &'static str {
    if symbol == "battery.100" {
        if let Some(soc) =
            value.and_then(|value| value.trim().trim_end_matches('%').parse::<f64>().ok())
        {
            let soc = soc.clamp(0.0, 100.0);
            return if soc >= 80.0 {
                "#22C55E"
            } else if soc >= 60.0 {
                "#84CC16"
            } else if soc >= 40.0 {
                "#EAB308"
            } else {
                "#EF4444"
            };
        }
    }
    match symbol {
        "house.fill" => "#BFD42D",
        "sun.max.fill" => "#42B9F5",
        _ => "white",
    }
}

#[cfg(target_os = "windows")]
fn bolt_path(width: i32) -> &'static str {
    if width <= 16 {
        r#"<path d="M13 1 6 8h4l-4 7 9-8h-4z" fill="white"/>"#
    } else {
        r#"<path d="M26 2 12 16h8l-8 14 18-16h-8z" fill="white"/>"#
    }
}

#[cfg(not(target_os = "windows"))]
fn bolt_path(_width: i32) -> &'static str {
    r#"<path d="M19 1 9 9h6l-5 8 14-10h-6z" fill="white"/>"#
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn compact_value(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('%')
        .replace(" kW", "k")
        .replace(" W", "")
}

#[cfg(test)]
mod tests {
    use super::{compact_value, escape_xml};

    #[test]
    fn compact_values_fit_the_tray_icon() {
        assert_eq!(compact_value("78%"), "78");
        assert_eq!(compact_value("1.2 kW"), "1.2k");
        assert_eq!(compact_value("850 W"), "850");
    }

    #[test]
    fn values_are_safe_for_svg_text() {
        assert_eq!(escape_xml("<&>\"'"), "&lt;&amp;&gt;&quot;&apos;");
    }
}
