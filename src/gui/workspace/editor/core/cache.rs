use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::editor::Editor;

impl Editor {
    pub(in crate::gui::workspace) fn touch_editor(&mut self, file: &ProjectFile) {
        self.state.editor_order.retain(|cached| cached != file);
        self.state.editor_order.push(file.clone());
    }

    pub(in crate::gui::workspace) fn prune_editors(&mut self) {
        let candidates: Vec<_> = self
            .state
            .editor_order
            .iter()
            .filter(|file| self.state.editors.contains_key(*file))
            .filter(|file| {
                DATA_CONTEXT
                    .documents
                    .snapshot(&file.path(DATA_CONTEXT.project_root()))
                    .is_some_and(|document| !document.dirty() && !document.saving)
            })
            .cloned()
            .collect();
        let mut released = 0;
        for file in candidates {
            if self.state.open_files.contains(&file)
                || self.state.loading_files.contains(&file)
                || self.state.saving_files.contains(&file)
            {
                continue;
            }
            if APPLICATION
                .documents
                .release_clean(&file.path(DATA_CONTEXT.project_root()))
            {
                self.state.editor_subscriptions.remove(&file);
                self.state.editors.remove(&file);
                self.state.external_source_files.remove(&file);
                self.state.editor_revisions.remove(&file);
                self.state.editor_order.retain(|cached| *cached != file);
                released += 1;
                tracing::debug!(file = %file.relative_path().display(), "clean document and editor cache released");
            }
        }
        if released > 0 {
            tracing::info!(
                released,
                cached = self.state.editors.len(),
                "closed clean editors released"
            );
        }
    }
}
