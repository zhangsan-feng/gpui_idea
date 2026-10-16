use super::*;
impl BottomBar {
    pub(super) fn peer_editor(&self, cx: &App) -> Entity<crate::gui::workspace::editor::Editor> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .editor
            .clone()
    }
}

impl BottomBar {
    pub(super) fn active_document(
        &self,
        cx: &App,
    ) -> Option<std::sync::Arc<crate::domain::document::DocumentSnapshot>> {
        let editor = self.peer_editor(cx);
        let state = &editor.read(cx).state;
        if state.open_files.is_empty() {
            None
        } else {
            DATA_CONTEXT
                .documents
                .snapshot(&state.selected_file.path(DATA_CONTEXT.project_root()))
        }
    }
    pub(super) fn sync_cursor(&mut self, cx: &mut Context<Self>) {
        let editor = self.peer_editor(cx);
        let state = &editor.read(cx).state;
        let editor = state.editors.get(&state.selected_file).cloned();
        let id = editor.as_ref().map(|e| e.entity_id());
        if self.state.cursor_editor != id {
            self.state.cursor_editor = id;
            self.state.cursor_subscription =
                editor.map(|editor| cx.observe(&editor, |_, _, cx| cx.notify()));
        }
    }
}
