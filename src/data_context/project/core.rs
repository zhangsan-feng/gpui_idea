use super::ProjectContext;
use crate::domain::project::DirectorySnapshot;
use std::{path::PathBuf, sync::Arc};

impl ProjectContext {
    pub(super) fn replace(&self, path: PathBuf, snapshot: DirectorySnapshot) {
        self.replace_silent(path, snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }

    pub(super) fn replace_silent(&self, path: PathBuf, snapshot: DirectorySnapshot) {
        {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            state.directories.insert(path, Arc::new(snapshot));
            state.revision += 1;
        }
    }
}
