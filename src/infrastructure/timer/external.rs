use super::{Command, PipelineState, TimerPipeline, TimerRegistration, Work, core};
use anyhow::{Context, Result, bail};
use std::{
    future::Future,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

impl TimerPipeline {
    pub(crate) fn start(&self) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.is_some() {
            return Ok(());
        }
        tokio::runtime::Handle::try_current().context("定时任务管线必须在 Tokio runtime 中启动")?;
        let (commands, receiver) = mpsc::unbounded_channel();
        *state = Some(PipelineState {
            commands,
            task: tokio::spawn(core::run(receiver)),
        });
        tracing::info!("timer pipeline started");
        Ok(())
    }
    fn register(
        &self,
        name: &'static str,
        delay: Duration,
        work: Work,
    ) -> Result<TimerRegistration> {
        let commands = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .as_ref()
            .context("定时任务管线尚未启动")?
            .commands
            .clone();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        commands
            .send(Command::Add {
                id,
                name,
                delay,
                work,
            })
            .context("定时任务管线已经关闭")?;
        Ok(TimerRegistration { id, commands })
    }
    pub(crate) async fn delay(&self, name: &'static str, delay: Duration) -> Result<()> {
        let (wake, receiver) = oneshot::channel();
        let _registration = self.register(name, delay, Work::Delay(wake))?;
        receiver.await.context("定时任务被取消或管线已经关闭")
    }
    pub(crate) fn every<F, Fut>(
        &self,
        name: &'static str,
        period: Duration,
        operation: F,
    ) -> Result<TimerRegistration>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        if period.is_zero() {
            bail!("定时任务周期不能为零");
        }
        self.register(
            name,
            period,
            Work::Repeating {
                period,
                operation: Arc::new(move || Box::pin(operation())),
            },
        )
    }
    pub(crate) async fn shutdown(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();
        if let Some(state) = state {
            state.task.abort();
            let _ = state.task.await;
            tracing::info!("timer pipeline shutdown complete");
        }
    }
}
