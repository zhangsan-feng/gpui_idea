use super::core;
use anyhow::Result;
use gpui_kit::App;
use std::path::Path;

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
    super::completion::attach(editor, core::language_for_path(path), window, cx);
}

pub(crate) fn completion_dropdown(
    editor: &gpui_kit::Entity<gpui_kit::component::input::EditorState>,
    content: impl gpui_kit::IntoElement,
    cx: &App,
) -> gpui_kit::Div {
    super::completion::wrap(editor, content, cx)
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
