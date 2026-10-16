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
        }
    }
}
