use crate::data_context::DATA_CONTEXT;

use anyhow::Result;
use gpui_kit::{
    App, AppContext as _, SharedString, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowOptions, px, size,
};

pub fn open_workspace(cx: &mut App) -> Result<()> {
    tracing::info!("main window opening: title=GPUI Editor");
    let minimum_size = size(px(1200.), px(700.));
    let saved_view = DATA_CONTEXT
        .startup()
        .session
        .as_ref()
        .and_then(|session| session.window_view);
    let window_size = saved_view
        .map(|view| {
            size(
                px(view.width).max(minimum_size.width),
                px(view.height).max(minimum_size.height),
            )
        })
        .unwrap_or(minimum_size);
    let centered_bounds = WindowBounds::centered(window_size, cx).get_bounds();
    let window_bounds = if saved_view.is_some_and(|view| view.maximized) {
        WindowBounds::Maximized(centered_bounds)
    } else {
        WindowBounds::Windowed(centered_bounds)
    };
    tracing::info!(
        ?window_size,
        maximized = saved_view.is_some_and(|view| view.maximized),
        "main window size prepared"
    );
    let options = WindowOptions {
        window_bounds: Some(window_bounds),
        window_min_size: Some(minimum_size),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from("")),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_decorations: Some(WindowDecorations::Client),
        ..WindowOptions::default()
    };
    gpui_kit::open_window(options, cx, move |window, cx| {
        cx.bind_keys([gpui_kit::KeyBinding::new(
            "ctrl-s",
            super::workspace::SaveDocument,
            Some("Workspace"),
        )]);
        let workspace = cx.new(|cx| super::workspace::Workspace::new(window, cx));
        let weak = workspace.downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            tracing::info!("main window native close requested: action=remove_window");
            if let Some(workspace) = weak.upgrade() {
                workspace.update(cx, |this, cx| this.request_close(window, cx));
            }
            false
        });
        workspace
    })?;
    tracing::info!("main window opened");
    Ok(())
}

pub(crate) fn open_tool_window<V: gpui_kit::Render>(
    title: &'static str,
    dimensions: gpui_kit::Size<gpui_kit::Pixels>,
    parent: &gpui_kit::Window,
    cx: &mut App,
    build: impl FnOnce(&mut gpui_kit::Window, &mut App) -> gpui_kit::Entity<V>,
) -> Result<(gpui_kit::AnyWindowHandle, gpui_kit::Entity<V>)> {
    let dimensions = size(
        dimensions.width,
        dimensions.height + px(crate::component::operation_title_bar::HEIGHT),
    );
    let parent_bounds = parent.bounds();
    let origin = gpui_kit::point(
        parent_bounds.origin.x + (parent_bounds.size.width - dimensions.width) / 2.,
        parent_bounds.origin.y + (parent_bounds.size.height - dimensions.height) / 2.,
    );
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(gpui_kit::Bounds {
            origin,
            size: dimensions,
        })),
        window_min_size: Some(dimensions),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_decorations: Some(WindowDecorations::Client),
        kind: gpui_kit::WindowKind::Dialog,
        is_resizable: false,
        is_minimizable: false,
        focus: true,
        ..WindowOptions::default()
    };
    tracing::info!(title, "operation dialog opening with custom title bar");
    gpui_kit::open_window(options, cx, build)
}
