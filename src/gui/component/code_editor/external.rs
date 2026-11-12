use super::core;
use anyhow::Result;
use gpui_kit::App;
use std::path::Path;

pub(crate) fn offset_at_point(
    editor: &gpui_kit::component::input::EditorState,
    pointer: gpui_kit::Point<gpui_kit::Pixels>,
) -> Option<usize> {
    super::drop_position::offset_at_point(editor, pointer)
}

pub(crate) fn gutter_layout(
    lines: usize,
    window: &mut gpui_kit::Window,
    cx: &App,
) -> super::GutterLayout {
    core::gutter_layout(lines, window, cx)
}

pub(crate) fn update_gutter(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    window: &mut gpui_kit::Window,
    cx: &mut App,
) {
    use gpui_kit::component::input::RopeExt as _;
    let layout = core::gutter_layout(editor.read(cx).text().lines_len(), window, cx);
    editor.update(cx, |state, cx| {
        state.set_gutter_decorations_width(layout.decorations_width, cx)
    });
}

pub(crate) fn definition_pointer(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    hitbox: &gpui_kit::Hitbox,
    window: &mut gpui_kit::Window,
    cx: &mut App,
) {
    super::definition_cursor::install(editor, hitbox, window, cx);
}

pub(crate) fn init(cx: &mut App) -> Result<()> {
    super::completion::install_keys(cx);
    core::install(cx)
}

pub(crate) fn attach_completions(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    path: &Path,
    window: &mut gpui_kit::Window,
    cx: &mut App,
) {
    editor.update(cx, |state, cx| core::configure_gutter(state, window, cx));
    super::highlights::attach(editor, path.to_owned(), window, cx);
    super::completion::attach(
        editor,
        core::language_for_path(path),
        path.to_owned(),
        window,
        cx,
    );
}

pub(crate) fn completion_dropdown(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    content: impl gpui_kit::IntoElement,
    cx: &App,
) -> gpui_kit::Div {
    use gpui_kit::InteractiveElement as _;

    let cut_editor = editor.clone();
    super::completion::wrap(editor, content, cx).capture_action(
        move |_: &gpui_kit::component::input::Cut, _, cx| {
            cut_editor.update(cx, core::select_line_for_cut);
            // Continue dispatching Cut to the component for clipboard and undo handling.
        },
    )
}

pub(crate) fn dismiss_completions(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    cx: &mut App,
) {
    super::completion::dismiss(editor, cx);
}

pub(crate) fn language_for_path(path: &Path) -> &'static str {
    core::language_for_path(path)
}

pub(crate) fn language_label(path: &Path) -> &'static str {
    core::language_definition(path).map_or("纯文本", |language| language.label)
}
