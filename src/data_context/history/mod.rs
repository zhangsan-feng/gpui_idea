mod core;
mod external;

use crate::domain::history::HistoryDataContextSnapshot;
use std::sync::{Arc, Mutex};

pub struct HistoryContext {
    page_override: Mutex<Option<crate::domain::history::SettingsPage>>,
    snapshot: Mutex<Arc<HistoryDataContextSnapshot>>,
}

impl HistoryContext {
    pub(super) fn new() -> Self {
        Self {
            page_override: Mutex::new(None),
            snapshot: Mutex::new(Arc::new(HistoryDataContextSnapshot::default())),
        }
    }
}
