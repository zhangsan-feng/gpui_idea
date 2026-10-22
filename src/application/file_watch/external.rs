use super::{FileWatchService, FileWatchTask, core};
use crate::{data_context::DATA_CONTEXT, event_bus::EVENT_BUS, infrastructure::INFRASTRUCTURE};
use anyhow::Result;

impl FileWatchService {
    pub(crate) async fn lock_refresh(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.refresh_lock.lock().await
    }

    /// Reload immediately and return after DataContext has been updated.
    pub(crate) async fn refresh_data_context(&self, changes: &crate::event_bus::FileChanges) {
        core::refresh(changes).await;
    }

    pub(crate) async fn start(&self) -> Result<()> {
        if !DATA_CONTEXT.has_project() {
            return Ok(());
        }
        let mut task = self.task.lock().await;
        if task.is_some() {
            return Ok(());
        }
        let root = DATA_CONTEXT.project_root().to_owned();
        // Subscribe before starting the source, so the first batch cannot be missed.
        let events = EVENT_BUS.subscribe_refresh_data_context();
        let subscription = INFRASTRUCTURE
            .filesystem
            .watch_project(root.clone())
            .await?;
        *task = Some(FileWatchTask {
            _subscription: subscription,
            consumer: Some(tokio::spawn(core::consume(root, events))),
        });
        tracing::info!("application subscribed to data context refresh requests");
        Ok(())
    }

    pub(crate) async fn shutdown(&self) {
        let task = self.task.lock().await.take();
        if let Some(mut task) = task {
            // Stop the source and wait for the consumer to finish cancellation before exit.
            let consumer = task.consumer.take();
            drop(task);
            if let Some(consumer) = consumer {
                consumer.abort();
                let _ = consumer.await;
            }
        }
        tracing::info!("filesystem event bus subscription stopped");
    }
}
