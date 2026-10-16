use anyhow::Result;
use std::{path::PathBuf, time::Instant};

pub(in crate::infrastructure::filesystem) async fn run<T: Send + 'static>(
    operation: &'static str,
    path: PathBuf,
    work: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    let started = Instant::now();
    let result = tokio::task::spawn_blocking(work).await?;
    tracing::debug!(operation, path = %path.display(), elapsed_ms = started.elapsed().as_millis(), success = result.is_ok(), "filesystem operation completed");
    if let Err(error) = &result {
        tracing::error!(operation, path = %path.display(), error = %error, "filesystem operation failed");
    }
    result
}
