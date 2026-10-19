use super::{FileChanges, FileSubscription, local_changes};
use anyhow::{Context, Result};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::{
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

pub(super) fn start(root: PathBuf) -> Result<FileSubscription> {
    let canonical_root = root.canonicalize().context("无法监听项目目录")?;
    let pending = Arc::new(Mutex::new(FileChanges::default()));
    local_changes::register(canonical_root.clone(), root.clone(), &pending);
    let events = pending.clone();
    let watched_root = canonical_root.clone();
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        match result {
            Ok(event) => {
                if matches!(event.kind, EventKind::Access(_)) {
                    return;
                }
                let rescan = event.need_rescan();
                let tree_changed = !matches!(event.kind,
                    EventKind::Modify(notify::event::ModifyKind::Data(_))
                    | EventKind::Modify(notify::event::ModifyKind::Metadata(_)));
                let paths: Vec<_> = event
                    .paths
                    .into_iter()
                    .filter_map(|path| {
                        let relative = path.strip_prefix(&watched_root).ok()?;
                        if ignored(relative) {
                            None
                        } else if local_changes::excludes(&path) {
                            tracing::debug!(path = %path.display(), "filesystem watcher ignored local change");
                            None
                        } else {
                            Some(root.join(relative))
                        }
                    })
                    .collect();
                if paths.is_empty() && !rescan {
                    return;
                }
                let mut batch = events.lock().unwrap_or_else(|error| error.into_inner());
                batch.rescan |= rescan;
                if !batch.rescan {
                    if tree_changed {
                        batch.tree_paths.extend(paths.iter().cloned());
                    }
                    batch.paths.extend(paths);
                    if batch.paths.len() > 4096 {
                        batch.rescan = true;
                    }
                }
                if batch.rescan {
                    batch.paths.clear();
                    batch.tree_paths.clear();
                }
                batch.event_count = batch.event_count.saturating_add(1);
            }
            Err(error) => {
                tracing::warn!(%error, "filesystem watcher error; scheduling full refresh");
                let mut batch = events.lock().unwrap_or_else(|error| error.into_inner());
                batch.rescan = true;
                batch.paths.clear();
                batch.tree_paths.clear();
                batch.event_count = batch.event_count.saturating_add(1);
            }
        }
        // Native callbacks only accumulate events; the timer pipeline drains each batch.
    })
    .context("无法启动文件系统监听")?;
    watcher
        .watch(&canonical_root, RecursiveMode::Recursive)
        .context("无法监听项目目录")?;
    tracing::info!(root = %canonical_root.display(), "filesystem watcher started");
    Ok(FileSubscription {
        _batch_timer: None,
        _watcher: watcher,
        pending,
    })
}

fn ignored(path: &Path) -> bool {
    path.components().any(|part| {
        matches!(part,
            Component::Normal(name) if matches!(name.to_str(), Some(
                ".git" | "target" | "node_modules" | "dist" | "build" | "artifacts"
                | "logs" | ".codex" | ".agents" | ".venv" | "__pycache__" | ".gpui-run"
            ))
        )
    })
}

pub(super) async fn register_gui_file_change(
    root: PathBuf,
    paths: Vec<PathBuf>,
) -> Result<super::LocalChangeGuard> {
    super::super::core::run("begin_gui_file_change", root.clone(), move || {
        let paths = paths
            .iter()
            .map(|path| super::super::core::path::resolve(&root, path, true))
            .collect::<Result<Vec<_>>>()?;
        Ok(local_changes::begin(paths))
    })
    .await
}

pub(super) fn take_watch_changes(pending: &Mutex<FileChanges>) -> Option<FileChanges> {
    local_changes::reconcile_expired();
    let mut pending = pending.lock().unwrap_or_else(|error| error.into_inner());
    if pending.event_count == 0 {
        return None;
    }
    Some(std::mem::take(&mut *pending))
}
