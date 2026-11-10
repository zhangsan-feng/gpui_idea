use super::ToolWindowOptions;
use gpui_kit::*;

pub(crate) fn open<V: Render>(
    options: ToolWindowOptions,
    parent: &Window,
    cx: &mut App,
    build: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
) -> anyhow::Result<(AnyWindowHandle, Entity<V>)> {
    let with_title = |dimensions: Size<Pixels>| {
        size(
            dimensions.width,
            dimensions.height + px(super::super::operation_title_bar::HEIGHT),
        )
    };
    let dimensions = with_title(options.dimensions);
    let parent_bounds = parent.bounds();
    let title = options.title;
    let window_options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: point(
                parent_bounds.origin.x + (parent_bounds.size.width - dimensions.width) / 2.,
                parent_bounds.origin.y + (parent_bounds.size.height - dimensions.height) / 2.,
            ),
            size: dimensions,
        })),
        window_min_size: Some(with_title(options.minimum)),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: None,
        }),
        window_decorations: Some(WindowDecorations::Client),
        kind: if options.resizable {
            WindowKind::Normal
        } else {
            WindowKind::Dialog
        },
        is_resizable: options.resizable,
        is_minimizable: false,
        focus: true,
        ..WindowOptions::default()
    };
    tracing::info!(
        title,
        resizable = options.resizable,
        "tool window opening with shared frame"
    );
    gpui_kit::open_window(window_options, cx, build)
}
