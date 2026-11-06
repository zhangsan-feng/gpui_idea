mod external;
mod file_tree;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct ProjectContext {
    pub file_tree: file_tree::FileTreeContext,
    state: Mutex<ProjectState>,
}
struct ProjectState {
    git_diff: Arc<crate::domain::git::GitDiffSnapshot>,
    content_search: Arc<crate::domain::project::ContentSearchDataContextSnapshot>,
    run_configuration: Arc<crate::domain::run::RunConfigurationDataContextSnapshot>,
    run_sessions: Arc<crate::domain::run::RunSessionsDataContextSnapshot>,
    git: Arc<crate::domain::git::GitDataContextSnapshot>,
    git_details: HashMap<String, Arc<crate::domain::git::GitDetailsDataContextSnapshot>>,
    statistics: Arc<crate::domain::statistics::StatisticsDataContextSnapshot>,
}
impl ProjectContext {
    pub(super) fn new() -> Self {
        Self {
            file_tree: file_tree::FileTreeContext::new(),
            state: Mutex::new(ProjectState {
                git_diff: Arc::default(),
                content_search: Arc::default(),
                run_configuration: Arc::default(),
                run_sessions: Arc::default(),
                git: Arc::default(),
                git_details: HashMap::new(),
                statistics: Arc::default(),
            }),
        }
    }
}
