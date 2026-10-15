use super::ProjectContext;
use crate::domain::project::DirectorySnapshot;
use std::{collections::HashMap, path::PathBuf, sync::Arc};

impl ProjectContext {
    pub fn content_search(&self) -> Arc<crate::domain::project::ContentSearchSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .content_search
            .clone()
    }
    pub(crate) fn set_content_search(
        &self,
        snapshot: crate::domain::project::ContentSearchSnapshot,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if snapshot.request < state.content_search.request {
            return;
        }
        state.content_search = Arc::new(snapshot);
        drop(state);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub fn run_sessions(&self) -> Arc<crate::domain::run::RunSessions> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_sessions
            .clone()
    }
    pub(crate) fn set_run_sessions(&self, snapshot: crate::domain::run::RunSessions) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_sessions = Arc::new(snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub fn run_configuration(&self) -> Arc<crate::domain::run::RunConfiguration> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_configuration
            .clone()
    }
    pub(crate) fn set_run_configuration(&self, snapshot: crate::domain::run::RunConfiguration) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if snapshot.request < state.run_configuration.request {
            return;
        }
        state.run_configuration = Arc::new(snapshot);
        drop(state);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub fn git(&self) -> Arc<crate::domain::git::GitSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git
            .clone()
    }
    pub(crate) fn set_git(&self, snapshot: crate::domain::git::GitSnapshot) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git = Arc::new(snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub fn git_details(&self, hash: &str) -> Arc<crate::domain::git::GitDetails> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git_details
            .get(hash)
            .cloned()
            .unwrap_or_default()
    }
    pub(crate) fn set_git_details(&self, snapshot: crate::domain::git::GitDetails) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git_details
            .insert(snapshot.hash.clone(), Arc::new(snapshot));
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub fn statistics(&self) -> Arc<crate::domain::statistics::StatisticsSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .statistics
            .clone()
    }
    pub(crate) fn set_statistics(&self, snapshot: crate::domain::statistics::StatisticsSnapshot) {
        self.set_statistics_silent(snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
    pub(crate) fn set_statistics_silent(
        &self,
        snapshot: crate::domain::statistics::StatisticsSnapshot,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .statistics = Arc::new(snapshot);
    }
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
    pub(crate) fn set_directory_silent(&self, path: PathBuf, snapshot: DirectorySnapshot) {
        self.replace_silent(path, snapshot);
    }
}
