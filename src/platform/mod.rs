pub(crate) mod startup;
pub(crate) mod tray;

#[cfg(target_os = "macos")]
pub(crate) fn configure_application_policy() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!("could not obtain the macOS main-thread marker");
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    if !app.setActivationPolicy(NSApplicationActivationPolicy::Accessory) {
        tracing::warn!("could not set macOS application activation policy to accessory");
    }
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn configure_application_policy() {}

mod instance;
pub(crate) use instance::InstanceLock;

#[cfg(target_os = "windows")]
pub(crate) fn hide_main_window(window: &gpui_kit::Window, _cx: &mut gpui_kit::App) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
    };

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        tracing::warn!("could not obtain the Windows window handle");
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        tracing::warn!("GPUI returned a non-Windows window handle");
        return;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn show_main_window(window: &gpui_kit::Window, _cx: &mut gpui_kit::App) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{ShowWindow, SW_SHOW},
    };

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        tracing::warn!("could not obtain the Windows window handle");
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        tracing::warn!("GPUI returned a non-Windows window handle");
        return;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn hide_main_window(_window: &gpui_kit::Window, cx: &mut gpui_kit::App) {
    // GPUI does not expose per-window hide/show yet. Hiding the accessory
    // application preserves the window entity and keeps the tray process
    // alive, so closing the dashboard is reversible from the tray menu.
    cx.hide();
}

#[cfg(target_os = "macos")]
pub(crate) fn show_main_window(_window: &gpui_kit::Window, cx: &mut gpui_kit::App) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;

    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        app.unhide(None);
    }
    cx.activate(true);
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(crate) fn hide_main_window(_window: &gpui_kit::Window, _cx: &mut gpui_kit::App) {}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(crate) fn show_main_window(_window: &gpui_kit::Window, _cx: &mut gpui_kit::App) {}

#[cfg(target_os = "windows")]
mod windows_tray_icon;
