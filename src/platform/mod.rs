pub(crate) mod tray;

#[cfg(target_os = "windows")]
pub(crate) fn hide_main_window(window: &gpui_kit::Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{ShowWindow, SW_HIDE},
    };

    let Ok(handle) = window.window_handle() else {
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
mod windows_tray_icon;
