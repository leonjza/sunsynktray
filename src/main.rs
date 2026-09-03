mod app;
mod assets;
mod diagnostics;
mod domain;
mod platform;
mod storage;
mod sunsynk;
mod ui;

use anyhow::Result;
use gpui::*;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let settings = storage::config::Settings::from_env()?;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--inspect-api") {
        return diagnostics::run(&args, settings);
    }
    Application::new()
        .with_assets(assets::Assets)
        .run(move |cx| {
            gpui_component::init(cx);
            platform::tray::install(cx);
            let state = app::MonitorState::new(settings.clone());
            cx.set_global(app::MonitorStateGlobal(state.clone()));
            app::open_main_window(cx, state);
        });
    Ok(())
}
