use super::ProjectContext;
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::Arc;

impl ProjectContext {
    pub(crate) fn set_content_search(
        &self,
        snapshot: crate::domain::project::ContentSearchDataContextSnapshot,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if snapshot.request < state.content_search.request {
            return;
        }
        state.content_search = Arc::new(snapshot);
        drop(state);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::FileTree]);
    }
    pub fn run_sessions(&self) -> Arc<crate::domain::run::RunSessionsDataContextSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_sessions
            .clone()
    }
    pub(crate) fn set_run_sessions(
        &self,
        snapshot: crate::domain::run::RunSessionsDataContextSnapshot,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_sessions = Arc::new(snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[
            GuiComponent::WindowBar,
            GuiComponent::LeftSidebar,
            GuiComponent::BottomPanel,
        ]);
    }
    pub fn run_configuration(
        &self,
    ) -> Arc<crate::domain::run::RunConfigurationDataContextSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .run_configuration
            .clone()
    }
    pub(crate) fn set_run_configuration(
        &self,
        snapshot: crate::domain::run::RunConfigurationDataContextSnapshot,
    ) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if snapshot.request < state.run_configuration.request {
            return;
        }
        state.run_configuration = Arc::new(snapshot);
        drop(state);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[
            GuiComponent::WindowBar,
            GuiComponent::BottomPanel,
            GuiComponent::RunConfiguration,
        ]);
    }
    pub fn git(&self) -> Arc<crate::domain::git::GitDataContextSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git
            .clone()
    }
    pub(crate) fn set_git(&self, snapshot: crate::domain::git::GitDataContextSnapshot) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git = Arc::new(snapshot);
        crate::data_context::DATA_CONTEXT
            .notify_gui_refresh(&[GuiComponent::WindowBar, GuiComponent::BottomPanel]);
    }
    pub fn git_details(
        &self,
        hash: &str,
    ) -> Arc<crate::domain::git::GitDetailsDataContextSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git_details
            .get(hash)
            .cloned()
            .unwrap_or_default()
    }
    pub(crate) fn set_git_details(
        &self,
        snapshot: crate::domain::git::GitDetailsDataContextSnapshot,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .git_details
            .insert(snapshot.hash.clone(), Arc::new(snapshot));
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::BottomPanel]);
    }
    pub fn statistics(&self) -> Arc<crate::domain::statistics::StatisticsDataContextSnapshot> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .statistics
            .clone()
    }
    pub(crate) fn set_statistics(
        &self,
        snapshot: crate::domain::statistics::StatisticsDataContextSnapshot,
    ) {
        self.set_statistics_silent(snapshot);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::BottomPanel]);
    }
    pub(crate) fn set_statistics_silent(
        &self,
        snapshot: crate::domain::statistics::StatisticsDataContextSnapshot,
    ) {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .statistics = Arc::new(snapshot);
    }
}
