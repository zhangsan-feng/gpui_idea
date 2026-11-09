use super::HistoryContext;
use crate::domain::history::HistoryDataContextSnapshot;
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::Arc;

impl HistoryContext {
    pub(super) fn replace(&self, mut history: HistoryDataContextSnapshot) {
        // A background project/session publication must not roll back newer navigation.
        let page = self
            .page_override
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(page) = *page {
            history.settings_page = page;
        }
        *self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(history);
        drop(page);
        crate::data_context::DATA_CONTEXT
            .notify_gui_refresh(&[GuiComponent::ProjectManagerWindow, GuiComponent::WindowBar]);
    }

    pub(super) fn remember_settings_page_core(&self, page: crate::domain::history::SettingsPage) {
        let mut current_page = self
            .page_override
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *current_page = Some(page);
        let mut snapshot = self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut next = (**snapshot).clone();
        next.settings_page = page;
        *snapshot = Arc::new(next);
    }
}
