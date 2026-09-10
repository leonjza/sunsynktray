use super::{Dashboard, MonitorController, MonitorState};
use gpui_kit::*;
use std::sync::{Arc, Mutex};

pub(crate) fn open_main_window(
    cx: &mut App,
    state: Arc<MonitorState>,
    controller: Entity<MonitorController>,
    show: bool,
) {
    let bounds = Bounds::centered(None, size(px(620.), px(840.)), cx);
    if let Err(error) = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::component::TitleBar::title_bar_options()),
            is_resizable: true,
            focus: show,
            show,
            // The dashboard body is scrollable, and compact mode needs to be
            // able to resize the native window below the full dashboard size.
            window_min_size: Some(size(px(560.), px(360.))),
            ..Default::default()
        },
        |window, cx| {
            #[cfg(any(target_os = "windows", target_os = "macos"))]
            window.on_window_should_close(cx, |window, _cx| {
                // Closing the main window hides SunTray to the tray instead of
                // destroying the window. The tray's Quit action remains the
                // explicit way to exit the application.
                crate::platform::hide_main_window(window, _cx);
                false
            });

            let view = cx.new(|cx| Dashboard::new(state, controller, window, cx));
            cx.new(|cx| gpui_kit::component::Root::new(view, window, cx))
        },
    ) {
        tracing::error!(%error, "failed to open SunTray window");
        return;
    }
    if show {
        cx.activate(true);
    }
}

pub(crate) fn open_connection_log_window(
    cx: &mut App,
    controller: Entity<MonitorController>,
    window_handle: Arc<Mutex<Option<AnyWindowHandle>>>,
) {
    if let Some(handle) = window_handle
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
        .copied()
    {
        if handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
        {
            return;
        }
        *window_handle
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
    }
    let bounds = Bounds::centered(None, size(px(900.), px(460.)), cx);
    let handle = match cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::component::TitleBar::title_bar_options()),
            is_resizable: true,
            focus: true,
            show: true,
            window_min_size: Some(size(px(640.), px(300.))),
            ..Default::default()
        },
        |window, cx| {
            let window_handle = window_handle.clone();
            window.on_window_should_close(cx, move |_, _| {
                *window_handle
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = None;
                true
            });
            let view =
                cx.new(|cx| crate::ui::connection_log::ConnectionLogView::new(controller, cx));
            cx.new(|cx| gpui_kit::component::Root::new(view, window, cx))
        },
    ) {
        Ok(handle) => handle,
        Err(error) => {
            tracing::error!(%error, "failed to open connection log window");
            return;
        }
    };
    // The handle is recorded only after the window has been created, so a
    // subsequent tray click can activate this exact window instead of opening
    // a duplicate.
    *window_handle
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(handle.into());
}
