use super::TimerPipeline;
use anyhow::Result;
use std::time::Duration;

impl TimerPipeline {
    pub(crate) fn start(&self) -> Result<()> {
        self.start_subscribe()
    }

    pub(crate) async fn delay(&self, name: &'static str, delay: Duration) -> Result<()> {
        crate::event_bus::EVENT_BUS
            .timer_event
            .delay(name, delay)
            .await
    }

    pub(crate) async fn shutdown(&self) {
        let task = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(task) = task {
            task.abort();
            let _ = task.await;
            tracing::info!("timer event subscription shutdown complete");
        }
    }
}
