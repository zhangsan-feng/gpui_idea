use super::HistoryContext;
use crate::domain::history::HistoryDataContextSnapshot;
use std::sync::Arc;

impl HistoryContext {
    pub(crate) fn remember_settings_page(&self, page: crate::domain::history::SettingsPage) {
        self.remember_settings_page_core(page);
    }

    pub fn snapshot(&self) -> Arc<HistoryDataContextSnapshot> {
        self.snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_history(&self, history: HistoryDataContextSnapshot) {
        self.replace(history);
    }
}
