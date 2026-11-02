mod core;
mod external;

use crate::domain::history::HistoryDataContextSnapshot;
use std::sync::{Arc, Mutex};

pub struct HistoryContext {
    snapshot: Mutex<Arc<HistoryDataContextSnapshot>>,
}

impl HistoryContext {
    pub(super) fn new() -> Self {
        Self {
            snapshot: Mutex::new(Arc::new(HistoryDataContextSnapshot::default())),
        }
    }
}
