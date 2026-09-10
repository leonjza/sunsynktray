pub(crate) mod startup;
pub(crate) mod tray;

pub(crate) fn app_data_dir() -> anyhow::Result<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .map(|path| path.join("Library/Application Support"));
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from);
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .map(|path| path.join(".local/share"))
        });

    base.map(|path| path.join("SunTray"))
        .ok_or_else(|| anyhow::anyhow!("could not determine the SunTray app-data directory"))
}

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
pub(crate) fn set_window_always_on_top(window: &gpui_kit::Window, enabled: bool) -> bool {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    };

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        tracing::warn!("could not obtain the Windows window handle");
        return false;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        tracing::warn!("GPUI returned a non-Windows window handle");
        return false;
    };
    let hwnd = HWND(handle.hwnd.get() as *mut _);
    let insert_after = if enabled {
        HWND_TOPMOST
    } else {
        HWND_NOTOPMOST
    };
    unsafe {
        SetWindowPos(
            hwnd,
            Some(insert_after),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .is_ok()
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn set_window_always_on_top(window: &gpui_kit::Window, enabled: bool) -> bool {
    use objc2_app_kit::{NSFloatingWindowLevel, NSNormalWindowLevel, NSView};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        tracing::warn!("could not obtain the macOS window handle");
        return false;
    };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        tracing::warn!("GPUI returned a non-AppKit window handle");
        return false;
    };

    // GPUI exposes the native NSView; AppKit provides the owning NSWindow.
    let view = unsafe { &*handle.ns_view.as_ptr().cast::<NSView>() };
    let Some(native_window) = view.window() else {
        tracing::warn!("could not find the AppKit window for the GPUI view");
        return false;
    };
    native_window.setLevel(if enabled {
        NSFloatingWindowLevel
    } else {
        NSNormalWindowLevel
    });
    true
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(crate) fn set_window_always_on_top(_window: &gpui_kit::Window, _enabled: bool) -> bool {
    false
}

#[cfg(target_os = "windows")]
mod windows_tray_icon;
