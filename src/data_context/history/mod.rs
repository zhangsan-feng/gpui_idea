mod core;
mod external;

use crate::domain::history::OpenHistory;
use std::sync::{Arc, Mutex};

pub struct HistoryContext {
    snapshot: Mutex<Arc<OpenHistory>>,
}

impl HistoryContext {
    pub(super) fn new() -> Self {
        Self {
            snapshot: Mutex::new(Arc::new(OpenHistory::default())),
        }
    }
}
