use super::core;
use crate::infrastructure::INFRASTRUCTURE;
use anyhow::Result;

pub fn initialize() -> Result<()> {
    core::initialize()?;
    tracing::info!("application services initialized");
    Ok(())
}

pub async fn prepare_startup() -> Result<()> {
    run_application(core::prepare_startup()).await
}

pub async fn run_application<T: Send + 'static>(
    operation: impl std::future::Future<Output = Result<T>> + Send + 'static,
) -> Result<T> {
    INFRASTRUCTURE.run(operation).await
}
