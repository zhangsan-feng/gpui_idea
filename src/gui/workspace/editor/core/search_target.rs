use super::super::*;
use crate::gui::workspace::editor::Editor;
impl Editor {
    pub(in crate::gui::workspace) fn open_content_match(
        &mut self,
        result: crate::domain::project::ContentMatch,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        use gpui_kit::component::input::{Position, RopeExt as _};
        let file = crate::gui::workspace::ProjectFile(result.path);
        let path = file.path(DATA_CONTEXT.project_root());
        self.state.pending_search_positions.insert(
            path,
            Position::new(
                result.line.saturating_sub(1),
                result.column.saturating_sub(1),
            ),
        );
        self.select_file(file.clone(), window, cx);
        if let Some(editor) = self.state.editors.get(&file) {
            self.state
                .pending_search_positions
                .remove(&file.path(DATA_CONTEXT.project_root()));
            editor.update(cx, |state, cx| {
                let position = Position::new(
                    result
                        .line
                        .saturating_sub(1)
                        .min(state.text().lines_len().saturating_sub(1) as u32),
                    result.column.saturating_sub(1),
                );
                state.set_cursor_position(position, window, cx);
            });
            Self::position_jump_viewport(editor, window, cx);
        }
    }

    pub(super) fn position_jump_viewport(
        editor: &Entity<EditorState>,
        window: &mut Window,
        cx: &mut App,
    ) {
        Self::position_jump_after_layout(editor, editor.read(cx).cursor(), 2, window);
    }

    fn position_jump_after_layout(
        editor: &Entity<EditorState>,
        cursor: usize,
        remaining_frames: u8,
        window: &mut Window,
    ) {
        let editor = editor.downgrade();
        // A newly opened document has no line metrics until its first layout.
        window.on_next_frame(move |window, cx| {
            let Some(editor) = editor.upgrade() else {
                return;
            };
            editor.update(cx, |state, cx| {
                if state.cursor() != cursor || !state.focus_handle(cx).is_focused(window) {
                    return;
                }
                let Some(line_height) = state.line_height() else {
                    if remaining_frames > 0 {
                        Self::position_jump_after_layout(
                            &cx.entity(),
                            cursor,
                            remaining_frames - 1,
                            window,
                        );
                    }
                    return;
                };
                let viewport_height = state.input_bounds().size.height;
                if viewport_height <= px(0.) {
                    return;
                }
                let line = state.cursor_position().line;
                let mut offset = state.scroll_offset();
                offset.y = (viewport_height / 3. - line_height * line as f32).min(px(0.));
                tracing::debug!(
                    line = line + 1,
                    viewport_height_px = f32::from(viewport_height),
                    offset_y_px = f32::from(offset.y),
                    "positioning source jump in upper third of editor"
                );
                state.set_scroll_offset(offset, cx);
            });
        });
    }
}
