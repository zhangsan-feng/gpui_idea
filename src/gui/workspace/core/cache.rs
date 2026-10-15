use super::super::{ProjectFile, Workspace};
use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;

const CLEAN_EDITOR_LIMIT: usize = 16;

impl Workspace {
    pub(in crate::gui::workspace) fn touch_editor(&mut self, file: &ProjectFile) {
        self.editor_order.retain(|cached| cached != file);
        self.editor_order.push(file.clone());
    }

    pub(in crate::gui::workspace) fn prune_editors(&mut self) {
        let candidates: Vec<_> = self
            .editor_order
            .iter()
            .filter(|file| self.editors.contains_key(*file))
            .filter(|file| {
                DATA_CONTEXT
                    .documents
                    .snapshot(&file.path(DATA_CONTEXT.project_root()))
                    .is_some_and(|document| !document.dirty() && !document.saving)
            })
            .cloned()
            .collect();
        let mut remaining = candidates.len().saturating_sub(CLEAN_EDITOR_LIMIT);
        for file in candidates {
            if remaining == 0 {
                break;
            }
            if self.open_files.contains(&file)
                || self.loading_files.contains(&file)
                || self.saving_files.contains(&file)
            {
                continue;
            }
            if APPLICATION
                .documents
                .release_clean(&file.path(DATA_CONTEXT.project_root()))
            {
                self.editor_subscriptions.remove(&file);
                self.editors.remove(&file);
                self.external_source_files.remove(&file);
                self.editor_revisions.remove(&file);
                self.editor_order.retain(|cached| *cached != file);
                remaining -= 1;
                tracing::debug!(file = %file.relative_path().display(), "clean document and editor cache released");
            }
        }
    }
}
