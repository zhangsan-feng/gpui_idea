mod application;
mod component;
mod data_context;
mod domain;
mod gui;
mod infrastructure;

use anyhow::Context as _;
use gpui_kit::component::scroll::ScrollbarMode;
use gpui_kit::component::{Theme, ThemeMode};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    let _logging_guard = tokio::task::spawn_blocking(|| {
        infrastructure::logging::initialize(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("logs"),
            "%Y-%m-%d",
        )
    })
    .await
    .context("initialize logging task failed")?;
    tracing::info!(application = "GPUI Editor", "application startup");

    if let Err(error) = application::lifecycle::initialize() {
        tracing::error!(error = %error, "application initialization failed");
        return Err(error);
    }
    if let Err(error) = application::lifecycle::prepare_startup().await {
        tracing::error!(error = %error, "application startup restoration failed");
        return Err(error);
    }
    // Keep GPUI on the main thread without inheriting this Tokio poll's coop budget.
    tokio::task::block_in_place(move || {
        tracing::info!(
            execution = "block_in_place",
            "GPUI application event loop starting"
        );
        gpui_kit::application()
            .with_assets(component::assets::MergedAssets::new())
            .run(move |cx| {
                gpui_kit::init(cx);
                Theme::change(ThemeMode::Light, None, cx);
                Theme::set_scrollbar_mode(ScrollbarMode::Always, cx);

                if let Err(error) = gui::external::open_workspace(cx) {
                    tracing::error!(error = %error, "failed to open the main window");
                    cx.quit();
                }
            });
    });

    tracing::info!("GPUI application event loop returned; shutdown starting");
    tracing::info!("application shutdown complete");
    Ok(())
}
