use super::super::FileSystem;
use crate::domain::statistics::StatisticsSnapshot;
use anyhow::Result;
use std::{path::PathBuf, time::Instant};

impl FileSystem {
    pub async fn statistics(&self, root: PathBuf) -> Result<StatisticsSnapshot> {
        let started = Instant::now();
        let result = super::core::scan(root.clone()).await;
        tracing::debug!(
            operation = "project_statistics",
            path = %root.display(),
            elapsed_ms = started.elapsed().as_millis(),
            success = result.is_ok(),
            "filesystem operation completed"
        );
        if let Err(error) = &result {
            tracing::error!(path = %root.display(), %error, "project statistics scan failed");
        }
        result
    }
}
