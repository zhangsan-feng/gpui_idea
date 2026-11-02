use super::CodeSearchWindow;
use crate::gui::workspace::Workspace;
use gpui_kit::{
    AnyWindowHandle, App, AppContext as _, TitlebarOptions, WeakEntity, Window, WindowBounds,
    WindowDecorations, WindowOptions, px, size,
};

pub(in crate::gui) fn open(
    parent: WeakEntity<Workspace>,
    window: &Window,
    cx: &mut App,
) -> anyhow::Result<AnyWindowHandle> {
    let parent_window = window.window_handle();
    let dimensions = size(px(780.), px(500.));
    let bounds = window.bounds();
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds {
            origin: gpui_kit::point(
                bounds.origin.x + (bounds.size.width - dimensions.width) / 2.,
                bounds.origin.y + (bounds.size.height - dimensions.height) / 2.,
            ),
            size: dimensions,
        })),
        window_min_size: Some(size(px(560.), px(360.))),
        titlebar: Some(TitlebarOptions {
            title: Some("代码内容搜索".into()),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_decorations: Some(WindowDecorations::Client),
        kind: gpui_kit::WindowKind::Normal,
        is_resizable: true,
        is_minimizable: false,
        focus: true,
        ..WindowOptions::default()
    };
    let (handle, _) = gpui_kit::open_window(options, cx, move |window, cx| {
        cx.new(|cx| CodeSearchWindow::new(parent, parent_window, window, cx))
    })?;
    tracing::info!("code search window opened");
    Ok(handle)
}
