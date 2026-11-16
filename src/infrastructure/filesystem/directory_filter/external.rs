use super::DirectoryFilter;
use crate::infrastructure::filesystem::FileSystem;
use std::sync::Arc;

impl FileSystem {
    pub(crate) fn set_ignored_directories(&self, names: Vec<String>) {
        *self
            .directory_filter
            .write()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(DirectoryFilter { names });
    }

    pub(in crate::infrastructure::filesystem) fn directory_filter(&self) -> Arc<DirectoryFilter> {
        self.directory_filter
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }
}
