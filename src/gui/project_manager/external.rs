use super::{ProjectManager, ProjectWindowKind};
use anyhow::Result;
use gpui_kit::{AnyWindowHandle, App, AppContext as _, px, size};
pub(crate) fn open(
    mode: ProjectWindowKind,
    parent: &gpui_kit::Window,
    cx: &mut App,
) -> Result<AnyWindowHandle> {
    let (window, _) = crate::gui::external::open_tool_window(
        mode.title(),
        size(px(640.), px(510.)),
        parent,
        cx,
        move |window, cx| cx.new(|cx| ProjectManager::new(mode, window, cx)),
    )?;
    Ok(window)
}
