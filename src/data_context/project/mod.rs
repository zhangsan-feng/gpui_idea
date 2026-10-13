mod core;
mod external;

use crate::domain::project::DirectorySnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

pub struct ProjectContext {
    state: Mutex<ProjectState>,
    gui_refresh_notification: watch::Sender<u64>,
}
struct ProjectState {
    directories: HashMap<PathBuf, Arc<DirectorySnapshot>>,
    revision: u64,
}
impl ProjectContext {
    pub(super) fn new(notification: watch::Sender<u64>) -> Self {
        Self {
            state: Mutex::new(ProjectState {
                directories: HashMap::new(),
                revision: 0,
            }),
            gui_refresh_notification: notification,
        }
    }
}
