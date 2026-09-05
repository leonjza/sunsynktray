#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod assets;
mod diagnostics;
mod domain;
mod platform;
mod storage;
mod sunsynk;
mod ui;

use anyhow::Result;
use gpui_kit::AppContext;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let settings = storage::config::Settings::from_env()?;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let startup = args.iter().any(|arg| arg == "--startup");
    if args.iter().any(|arg| arg == "--inspect-api") {
        return diagnostics::run(&args, settings);
    }
    let Some(_instance_lock) = platform::InstanceLock::acquire()? else {
        tracing::info!("another SunTray instance is already running");
        return Ok(());
    };
    gpui_kit::application()
        .with_assets(assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            platform::configure_application_policy();
            gpui_kit::component::Theme::sync_system_appearance(None, cx);
            platform::tray::install(cx);
            let state = app::MonitorState::new(settings.clone());
            cx.set_global(app::MonitorStateGlobal(state.clone()));
            let controller = cx.new(|_| app::MonitorController::new(state.clone()));
            cx.set_global(app::MonitorControllerGlobal(controller.clone()));
            controller.update(cx, |controller, cx| controller.initialize(cx));
            app::open_main_window(cx, state, controller, !startup);
        });
    Ok(())
}
