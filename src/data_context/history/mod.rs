mod core;
mod external;

use crate::domain::history::OpenHistory;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

pub struct HistoryContext {
    snapshot: Mutex<Arc<OpenHistory>>,
    gui_refresh_notification: watch::Sender<u64>,
}

impl HistoryContext {
    pub(super) fn new(notification: watch::Sender<u64>) -> Self {
        Self {
            snapshot: Mutex::new(Arc::new(OpenHistory::default())),
            gui_refresh_notification: notification,
        }
    }
}
