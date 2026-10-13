use super::HistoryContext;
use crate::domain::history::OpenHistory;
use std::sync::Arc;

impl HistoryContext {
    pub fn snapshot(&self) -> Arc<OpenHistory> {
        self.snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_history(&self, history: OpenHistory) {
        self.replace(history);
    }
}
