use super::HistoryContext;
use crate::domain::history::OpenHistory;
use std::sync::Arc;

impl HistoryContext {
    pub(super) fn replace(&self, history: OpenHistory) {
        *self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(history);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh();
    }
}
