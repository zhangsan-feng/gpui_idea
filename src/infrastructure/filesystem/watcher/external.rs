use super::{FileSubscription, core};
use crate::{event_bus::EVENT_BUS, infrastructure::filesystem::FileSystem};
use anyhow::Result;
use std::{path::PathBuf, sync::Mutex, time::Duration};

impl FileSystem {
    /// Application registers validated target paths before its own file operation.
    pub(crate) async fn begin_gui_file_change(
        &self,
        root: PathBuf,
        paths: Vec<PathBuf>,
    ) -> Result<super::LocalChangeGuard> {
        EVENT_BUS
            .timer_event
            .exclude_gui_file_change(root, paths)
            .await
    }

    /// Low-level handler called only after the timer subscriber receives the event.
    pub(in crate::infrastructure) async fn register_gui_file_change(
        &self,
        root: PathBuf,
        paths: Vec<PathBuf>,
    ) -> Result<super::LocalChangeGuard> {
        core::register_gui_file_change(root, paths).await
    }

    pub(in crate::infrastructure) fn take_watch_changes(
        &self,
        pending: &Mutex<super::FileChanges>,
    ) -> Option<super::FileChanges> {
        core::take_watch_changes(pending)
    }

    pub(crate) async fn watch_project(&self, root: PathBuf) -> Result<FileSubscription> {
        let watched_root = root.clone();
        let mut subscription = tokio::task::spawn_blocking(move || core::start(root)).await??;
        let pending = subscription.pending.clone();
        subscription._batch_timer = Some(EVENT_BUS.timer_event.watch_files(
            watched_root,
            pending,
            Duration::from_secs(3),
        )?);
        tracing::info!(
            check_ms = 3000,
            "filesystem batches publishing to event bus"
        );
        Ok(subscription)
    }
}
