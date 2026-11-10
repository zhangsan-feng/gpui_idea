use super::GitCreateWindow;
use gpui_kit::{AnyWindowHandle, App, AppContext as _, Window, px, size};

pub(in crate::gui) fn open(window: &Window, cx: &mut App) -> anyhow::Result<AnyWindowHandle> {
    let (handle, _) = crate::gui::external::open_tool_window(
        "创建 Git 分支",
        size(px(560.), px(320.)),
        window,
        cx,
        |window, cx| {
            let view = cx.new(|cx| GitCreateWindow::new(window, cx));
            let weak = view.downgrade();
            window.on_window_should_close(cx, move |_, cx| {
                weak.upgrade().is_none_or(|view| !view.read(cx).busy)
            });
            view
        },
    )?;
    Ok(handle)
}
