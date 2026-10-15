use super::{FileSubscription, core, local_changes};
use crate::{
    event_bus::EVENT_BUS,
    infrastructure::{INFRASTRUCTURE, filesystem::FileSystem},
};
use anyhow::Result;
use std::{path::PathBuf, time::Duration};

impl FileSystem {
    /// Application registers validated target paths before its own file operation.
    pub(crate) async fn begin_gui_file_change(
        &self,
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

    pub(crate) async fn watch_project(&self, root: PathBuf) -> Result<FileSubscription> {
        let watched_root = root.clone();
        let mut subscription = tokio::task::spawn_blocking(move || core::start(root)).await??;
        let pending = subscription.pending.clone();
        subscription._batch_timer = Some(INFRASTRUCTURE.timer.every(
            "refresh-data-context",
            Duration::from_secs(3),
            move || {
                let pending = pending.clone();
                let root = watched_root.clone();
                async move {
                    local_changes::reconcile_expired();
                    if let Some(changes) = core::take_changes(&pending) {
                        EVENT_BUS.refresh_data_context(root, changes);
                    }
                    Ok(())
                }
            },
        )?);
        tracing::info!(
            check_ms = 3000,
            "filesystem batches publishing to event bus"
        );
        Ok(subscription)
    }
}
