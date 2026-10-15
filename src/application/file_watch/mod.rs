mod core;
mod external;

use crate::infrastructure::FileSubscription;
use tokio::{sync::Mutex, task::JoinHandle};

pub(crate) struct FileWatchService {
    task: Mutex<Option<FileWatchTask>>,
    refresh_lock: Mutex<()>,
}

struct FileWatchTask {
    _subscription: FileSubscription,
    consumer: Option<JoinHandle<()>>,
}

impl Drop for FileWatchTask {
    fn drop(&mut self) {
        if let Some(consumer) = &self.consumer {
            consumer.abort();
        }
    }
}

impl FileWatchService {
    pub(super) fn new() -> Self {
        Self {
            task: Mutex::new(None),
            refresh_lock: Mutex::new(()),
        }
    }
}
