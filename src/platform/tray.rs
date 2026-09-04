#[cfg(target_os = "windows")]
use super::windows_tray_icon;
use gpui_kit::{actions, App, Global, MenuItem};
use gpui_tray::{Icon, Tray};

actions!(suntray_tray, [OpenDashboard, Quit]);

pub(crate) struct TrayState(pub Tray);
impl Global for TrayState {}

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
        Ok(tray) => cx.set_global(TrayState(tray)),
        Err(error) => tracing::error!("failed to create tray icon: {error}"),
    }
}

pub(crate) fn update(cx: &mut App, value: Option<&str>, symbol: &str, tooltip: &str) {
    let tray = cx.try_global::<TrayState>().map(|state| state.0.clone());
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
            if let Err(error) = tray.set_icon(Some(icon_for(value, cx, symbol)), cx) {
                tracing::error!("failed to update tray icon: {error}");
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
        let _ = window.update(cx, |_, window, _| window.activate_window());
    }
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

#[cfg(not(target_os = "windows"))]
fn icon_for(value: Option<&str>, cx: &App, _symbol: &str) -> Icon {
    #[cfg(target_os = "macos")]
    const FONT_FAMILY: &str = ".SF Compact";
    #[cfg(target_os = "windows")]
    const FONT_FAMILY: &str = "Segoe UI, sans-serif";
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    const FONT_FAMILY: &str = "Noto Sans, sans-serif";

    let body = match value {
        Some(value) => format!(
            r#"<text x="12" y="9" text-anchor="middle" dominant-baseline="middle" font-family="{FONT_FAMILY}" font-size="11" font-weight="600" fill="white">{value}</text>"#
        ),
        None => r#"<path d="M19 1 9 9h6l-5 8 14-10h-6z" fill="white"/>"#.into(),
    };
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="18" viewBox="0 0 24 18">{body}</svg>"#
    );
    let image = gpui_kit::Image::from_bytes(gpui_kit::ImageFormat::Svg, svg.into_bytes());
    Icon::from_gpui(&image, cx).expect("valid tray SVG")
}

#[cfg(target_os = "windows")]
fn icon_for(value: Option<&str>, _cx: &App, symbol: &str) -> Icon {
    windows_tray_icon::render(value, symbol).unwrap_or_else(|error| {
        tracing::warn!(%error, "could not render Windows tray icon");
        windows_tray_icon::fallback()
    })
}
