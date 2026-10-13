use super::ProjectContext;
use crate::domain::project::DirectorySnapshot;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

impl ProjectContext {
    pub fn revision(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .revision
    }
    pub fn snapshots(&self) -> (u64, HashMap<PathBuf, Arc<DirectorySnapshot>>) {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        (state.revision, state.directories.clone())
    }
    pub(crate) fn set_directory(&self, path: PathBuf, snapshot: DirectorySnapshot) {
        self.replace(path, snapshot);
    }
}
