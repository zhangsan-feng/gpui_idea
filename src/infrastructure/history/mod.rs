mod core;
mod external;

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub struct HistoryStore {
    path: PathBuf,
    io_lock: Arc<Mutex<()>>,
}

impl HistoryStore {
    pub fn new(path: PathBuf) -> Self {
        tracing::info!(path = %path.display(), "history store initialized");
        Self {
            path,
            io_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("open-history.json")
    }
}
