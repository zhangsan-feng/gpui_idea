mod core;
mod external;

use crate::domain::document::DocumentSnapshot;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

pub struct DocumentContext {
    documents: Mutex<HashMap<PathBuf, Arc<DocumentSnapshot>>>,
    gui_refresh_notification: watch::Sender<u64>,
}
impl DocumentContext {
    pub(super) fn new(notification: watch::Sender<u64>) -> Self {
        Self {
            documents: Mutex::new(HashMap::new()),
            gui_refresh_notification: notification,
        }
    }
}
