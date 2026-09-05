use super::{Dashboard, MonitorController, MonitorState};
use gpui_kit::*;
use std::sync::Arc;

pub(crate) fn open_main_window(
    cx: &mut App,
    state: Arc<MonitorState>,
    controller: Entity<MonitorController>,
    show: bool,
) {
    let bounds = Bounds::centered(None, size(px(620.), px(760.)), cx);
    if let Err(error) = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::component::TitleBar::title_bar_options()),
            is_resizable: true,
            focus: show,
            show,
            window_min_size: Some(size(px(560.), px(700.))),
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
