use super::HistoryContext;
use crate::domain::history::OpenHistory;
use std::sync::Arc;

impl HistoryContext {
    pub(super) fn replace(&self, history: OpenHistory) {
        *self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(history);
        self.gui_refresh_notification
            .send_modify(|version| *version += 1);
    }
}
