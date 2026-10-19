use super::FileRefreshBatch;
use crate::{data_context::DataContext, event_bus::refresh_gui::GuiComponent};

impl DataContext {
    pub(super) fn apply_file_refresh_core(&self, batch: FileRefreshBatch) {
        let mut editor_updated = batch.image_updated;
        for (expected, result) in batch.documents {
            editor_updated |= match result {
                Ok(loaded) => self.documents.refresh_from_disk_silent(&expected, loaded),
                Err(message) => self
                    .documents
                    .disk_refresh_failed_silent(&expected, message),
            };
        }
        let mut tree_updated = false;
        for (path, snapshot) in batch.directories {
            tree_updated |= self.project.set_directory_silent(path, snapshot);
        }
        let statistics_updated = batch.statistics.is_some();
        if let Some(snapshot) = batch.statistics {
            self.project.set_statistics_silent(snapshot);
        }
        let mut components = Vec::new();
        if editor_updated {
            components.extend([GuiComponent::Editor, GuiComponent::BottomBar]);
        }
        if tree_updated || editor_updated {
            // Rows also show dirty/error status for open documents.
            components.push(GuiComponent::FileTree);
        }
        if statistics_updated {
            components.push(GuiComponent::BottomPanel);
        }
        if !components.is_empty() {
            self.notify_gui_refresh(&components);
        }
        tracing::debug!(
            editor_updated,
            tree_updated,
            statistics_updated,
            "data context routed filesystem refresh"
        );
    }
}
