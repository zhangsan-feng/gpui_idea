use super::FileTreeContext;
use crate::domain::project::DirectoryDataContextSnapshot;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

impl FileTreeContext {
    pub fn is_code_package_directory(&self, path: &Path) -> bool {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        path.parent()
            .and_then(|parent| state.directories.get(parent))
            .is_some_and(|directory| {
                directory.entries.iter().any(|entry| {
                    entry.relative_path == path
                        && entry.is_directory
                        && !entry.is_link
                        && entry.is_code_package
                })
            })
    }

    pub fn is_file_runnable(&self, path: &Path) -> bool {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        path.parent()
            .and_then(|parent| state.directories.get(parent))
            .is_some_and(|directory| {
                directory.entries.iter().any(|entry| {
                    entry.relative_path == path
                        && entry.runnable
                        && !entry.is_directory
                        && !entry.is_link
                })
            })
    }

    pub fn revision(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .revision
    }

    pub fn snapshots(&self) -> (u64, HashMap<PathBuf, Arc<DirectoryDataContextSnapshot>>) {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        (state.revision, state.directories.clone())
    }

    pub(crate) fn set_directory(&self, path: PathBuf, snapshot: DirectoryDataContextSnapshot) {
        self.replace(path, snapshot);
    }

    pub(crate) fn set_directory_silent(
        &self,
        path: PathBuf,
        snapshot: DirectoryDataContextSnapshot,
    ) -> bool {
        self.replace_silent(path, snapshot)
    }
}
