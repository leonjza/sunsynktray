use super::{Dashboard, MonitorState};
use gpui_kit::*;
use std::sync::Arc;

pub(crate) fn open_main_window(cx: &mut App, state: Arc<MonitorState>) {
    let bounds = Bounds::centered(None, size(px(620.), px(760.)), cx);
    if let Err(error) = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::component::TitleBar::title_bar_options()),
            is_resizable: true,
            focus: true,
            show: true,
            window_min_size: Some(size(px(560.), px(700.))),
            ..Default::default()
        },
        |window, cx| {
            let view = cx.new(|cx| Dashboard::new(state, window, cx));
            cx.new(|cx| gpui_kit::component::Root::new(view, window, cx))
        },
    ) {
        tracing::error!(%error, "failed to open SunTray window");
        return;
    }
    cx.activate(true);
}
