mod core;
mod external;

use crate::domain::project::DirectorySnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct ProjectContext {
    state: Mutex<ProjectState>,
}
struct ProjectState {
    content_search: Arc<crate::domain::project::ContentSearchSnapshot>,
    run_configuration: Arc<crate::domain::run::RunConfiguration>,
    run_sessions: Arc<crate::domain::run::RunSessions>,
    git: Arc<crate::domain::git::GitSnapshot>,
    git_details: HashMap<String, Arc<crate::domain::git::GitDetails>>,
    statistics: Arc<crate::domain::statistics::StatisticsSnapshot>,
    directories: HashMap<PathBuf, Arc<DirectorySnapshot>>,
    revision: u64,
}
impl ProjectContext {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(ProjectState {
                content_search: Arc::default(),
                run_configuration: Arc::default(),
                run_sessions: Arc::default(),
                git: Arc::default(),
                git_details: HashMap::new(),
                statistics: Arc::default(),
                directories: HashMap::new(),
                revision: 0,
            }),
        }
    }
}
