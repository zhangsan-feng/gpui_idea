use super::{InfrastructureContext, runtime::AppRuntime};
use anyhow::{Result, anyhow};
use std::future::Future;

impl InfrastructureContext {
    pub(crate) fn initialize(&self) -> Result<()> {
        let runtime = AppRuntime::from_current()?;
        self.runtime
            .set(runtime)
            .map_err(|_| anyhow!("基础设施已经初始化"))?;
        tracing::info!("infrastructure runtime initialized");
        Ok(())
    }

    pub(crate) async fn run<T: Send + 'static>(
        &self,
        operation: impl Future<Output = Result<T>> + Send + 'static,
    ) -> Result<T> {
        self.runtime
            .get()
            .ok_or_else(|| anyhow!("基础设施尚未初始化"))?
            .run(operation)
            .await
    }
}
