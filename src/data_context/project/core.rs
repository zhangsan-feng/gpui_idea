use super::ProjectContext;
use crate::domain::project::DirectorySnapshot;
use crate::event_bus::refresh_gui::GuiComponent;
use std::{path::PathBuf, sync::Arc};

impl ProjectContext {
    pub(super) fn replace(&self, path: PathBuf, snapshot: DirectorySnapshot) {
        self.replace_silent(path, snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::FileTree]);
    }

    pub(super) fn replace_silent(&self, path: PathBuf, snapshot: DirectorySnapshot) -> bool {
        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if state
                .directories
                .get(&path)
                .is_some_and(|current| **current == snapshot)
            {
                return false;
            }
            state.directories.insert(path, Arc::new(snapshot));
            state.revision += 1;
        }
        true
    }
}
