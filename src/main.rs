#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod application;
mod data_context;
mod domain;
mod event_bus;
mod gui;
mod infrastructure;

use anyhow::Context as _;
use gpui_kit::component::scroll::ScrollbarMode;
use gpui_kit::component::{Theme, ThemeMode};
use gui::component;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let _logging_guard = tokio::task::spawn_blocking(|| {
        let paths = infrastructure::paths::initialize()?;
        Ok::<_, anyhow::Error>(infrastructure::logging::initialize(&paths.logs, "%Y-%m-%d"))
    })
    .await
    .context("initialize logging task failed")??;
    tracing::info!(root = %infrastructure::paths::get().root.display(), "application directories initialized");
    infrastructure::logging::log_memory("logging_initialized");
    tokio::task::spawn_blocking(|| {
        infrastructure::assets::initialize(&infrastructure::paths::get().assets)
    })
    .await
    .context("preload assets task failed")??;
    tracing::info!(application = "GPUI Editor", "application startup");
    infrastructure::logging::log_memory("assets_loaded");

    if let Err(error) = application::lifecycle::initialize() {
        tracing::error!(error = %error, "application initialization failed");
        return Err(error);
    }
    if let Err(error) = application::lifecycle::prepare_startup().await {
        tracing::error!(error = %error, "application startup restoration failed");
        return Err(error);
    }
    infrastructure::logging::log_memory("startup_prepared");
    // Keep GPUI on the main thread without inheriting this Tokio poll's coop budget.
    tokio::task::block_in_place(move || {
        tracing::info!(
            execution = "block_in_place",
            "GPUI application event loop starting"
        );
        gpui_kit::application()
            .with_assets(component::assets::MergedAssets::new())
            .run(move |cx| {
                infrastructure::logging::log_memory("platform_initialized");
                gpui_kit::init(cx);
                infrastructure::logging::log_memory("components_initialized");
                gui::component::terminal::init(cx);
                Theme::change(ThemeMode::Light, None, cx);
                Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);

                if let Err(error) = gui::external::open_workspace(cx) {
                    tracing::error!(error = %error, "failed to open the main window");
                    cx.quit();
                }
            });
    });

    tracing::info!("GPUI application event loop returned; shutdown starting");
    application::lifecycle::shutdown().await;
    tracing::info!("application shutdown complete");
    Ok(())
}
