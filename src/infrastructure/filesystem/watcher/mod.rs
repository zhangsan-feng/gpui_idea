mod core;
mod external;
mod local_changes;

pub(crate) use crate::event_bus::FileChanges;
use notify::RecommendedWatcher;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};

pub(crate) struct FileSubscription {
    _batch_timer: Option<crate::infrastructure::TimerRegistration>,
    _watcher: RecommendedWatcher,
    pending: Arc<Mutex<FileChanges>>,
}

struct LocalChange {
    id: u64,
    paths: Vec<PathBuf>,
    expires: Option<Instant>,
}

#[derive(Default)]
struct LocalChanges {
    next_id: u64,
    changes: Vec<LocalChange>,
    subscriptions: Vec<(PathBuf, Weak<Mutex<FileChanges>>)>,
}

pub(crate) struct LocalChangeGuard {
    id: Option<u64>,
}
