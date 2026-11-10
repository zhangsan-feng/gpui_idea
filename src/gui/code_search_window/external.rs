use super::CodeSearchWindow;
use crate::gui::component::tool_window::{self, ToolWindowOptions};
use crate::gui::workspace::Workspace;
use gpui_kit::{AnyWindowHandle, App, AppContext as _, WeakEntity, Window, px, size};

pub(in crate::gui) fn open(
    parent: WeakEntity<Workspace>,
    window: &Window,
    cx: &mut App,
) -> anyhow::Result<AnyWindowHandle> {
    let parent_window = window.window_handle();
    let title_height = crate::gui::component::operation_title_bar::HEIGHT;
    let options = ToolWindowOptions::new("代码内容搜索", size(px(780.), px(500. - title_height)))
        .resizable(size(px(560.), px(360. - title_height)));
    let (handle, _) = tool_window::open(options, window, cx, move |window, cx| {
        cx.new(|cx| CodeSearchWindow::new(parent, parent_window, window, cx))
    })?;
    tracing::info!("code search window opened");
    Ok(handle)
}
