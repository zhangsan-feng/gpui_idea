mod core;
mod external;

use crate::domain::project::DirectoryDataContextSnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct FileTreeContext {
    state: Mutex<FileTreeState>,
}

#[derive(Default)]
struct FileTreeState {
    directories: HashMap<PathBuf, Arc<DirectoryDataContextSnapshot>>,
    revision: u64,
}

impl FileTreeContext {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(FileTreeState::default()),
        }
    }
}
