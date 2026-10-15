use super::{FileChanges, LocalChange, LocalChangeGuard, LocalChanges};
use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

static LOCAL_CHANGES: LazyLock<Mutex<LocalChanges>> =
    LazyLock::new(|| Mutex::new(LocalChanges::default()));
const DELIVERY_GRACE: Duration = Duration::from_secs(1);

pub(super) fn register(root: PathBuf, pending: &Arc<Mutex<FileChanges>>) {
    let mut state = LOCAL_CHANGES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    state
        .subscriptions
        .retain(|(_, pending)| pending.strong_count() > 0);
    state.subscriptions.push((root, Arc::downgrade(pending)));
}

fn prune(state: &mut LocalChanges) {
    let now = Instant::now();
    state
        .changes
        .retain(|change| change.expires.is_none_or(|expires| expires > now));
}

// Register synchronously before IO starts; native callbacks use the same lock.
pub(super) fn begin(paths: Vec<PathBuf>) -> LocalChangeGuard {
    LocalChangeGuard {
        id: Some(apply_begin(paths)),
    }
}

pub(super) fn excludes(path: &Path) -> bool {
    let mut state = LOCAL_CHANGES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    prune(&mut state);
    state
        .changes
        .iter()
        .any(|change| change.paths.iter().any(|local| path.starts_with(local)))
}

fn apply_begin(paths: Vec<PathBuf>) -> u64 {
    let mut state = LOCAL_CHANGES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    prune(&mut state);
    state.next_id = state.next_id.wrapping_add(1);
    let id = state.next_id;
    tracing::debug!(
        id,
        paths = paths.len(),
        "local file change exclusion registered"
    );
    state.changes.push(LocalChange {
        id,
        paths,
        expires: None,
    });
    id
}

fn apply_finish(id: u64, failed: bool) {
    let subscriptions = {
        let mut state = LOCAL_CHANGES
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(index) = state.changes.iter().position(|change| change.id == id) else {
            return;
        };
        if !failed {
            state.changes[index].expires = Some(Instant::now() + DELIVERY_GRACE);
            tracing::debug!(id, "local change ended; excluding delayed native events");
            return;
        }
        let paths = state.changes.remove(index).paths;
        state
            .subscriptions
            .iter()
            .filter(|(root, _)| paths.iter().any(|path| path.starts_with(root)))
            .filter_map(|(_, pending)| pending.upgrade())
            .collect::<Vec<_>>()
    };
    // Partial IO may already have produced filtered events; reconcile the disk on failure.
    for pending in subscriptions {
        let mut pending = pending.lock().unwrap_or_else(|error| error.into_inner());
        pending.rescan = true;
        pending.paths.clear();
        pending.event_count = pending.event_count.saturating_add(1);
    }
    tracing::debug!(id, "local change failed; scheduling full refresh");
}

impl LocalChangeGuard {
    pub(crate) fn finish<T>(mut self, result: Result<T>) -> Result<T> {
        if let Some(id) = self.id.take() {
            apply_finish(id, result.is_err());
        }
        result
    }
}

impl Drop for LocalChangeGuard {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            // Cancellation or unwinding may leave partial changes on disk.
            apply_finish(id, true);
        }
    }
}
