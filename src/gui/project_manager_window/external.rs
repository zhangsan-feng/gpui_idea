use super::{ProjectManagerWindow, ProjectWindowKind};
use anyhow::Result;
use gpui_kit::{AnyWindowHandle, App, AppContext as _, px, size};
pub(crate) fn open(
    mode: ProjectWindowKind,
    parent: &gpui_kit::Window,
    cx: &mut App,
) -> Result<AnyWindowHandle> {
    let started = std::time::Instant::now();
    tracing::info!(mode = mode.title(), "project manager window opening");
    let (window, _) = crate::gui::external::open_tool_window(
        mode.title(),
        if mode == ProjectWindowKind::Create {
            size(px(840.), px(600.))
        } else {
            size(px(640.), px(510.))
        },
        parent,
        cx,
        move |window, cx| cx.new(|cx| ProjectManagerWindow::new(mode, window, cx)),
    )?;
    tracing::info!(
        mode = mode.title(),
        elapsed_ms = started.elapsed().as_millis(),
        "project manager window opened"
    );
    Ok(window)
}
