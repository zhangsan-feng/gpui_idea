use super::ProjectService;
use crate::application::APPLICATION;
use crate::domain::project::DirectorySnapshot;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail};
use std::{path::PathBuf, time::Instant};

impl ProjectService {
    pub(super) async fn move_core(
        &self,
        source: PathBuf,
        target_directory: PathBuf,
    ) -> Result<PathBuf> {
        let name = source
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("不能移动项目根目录"))?;
        let destination = target_directory.join(name);
        if source == destination {
            bail!("文件或目录已经位于目标目录");
        }
        if target_directory.starts_with(&source) {
            bail!("目录不能移动到自身或子目录中");
        }
        tracing::info!(source = %source.display(), destination = %destination.display(), "project entry move requested");
        INFRASTRUCTURE
            .filesystem
            .rename(
                DATA_CONTEXT.project_root().to_owned(),
                source,
                destination.clone(),
            )
            .await?;
        tracing::info!(destination = %destination.display(), "project entry moved");
        Ok(destination)
    }

    pub(super) async fn load_core(&self, relative: PathBuf) -> Result<()> {
        let _permit = self.reads.acquire().await?;
        let started = Instant::now();
        let result = INFRASTRUCTURE
            .filesystem
            .list_directory(DATA_CONTEXT.project_root().to_owned(), relative.clone())
            .await;
        if result.is_ok() {
            if let Err(error) = APPLICATION
                .history
                .record_opened(
                    crate::domain::history::OpenKind::Directory,
                    DATA_CONTEXT.project_root().join(&relative),
                )
                .await
            {
                tracing::warn!(directory = %relative.display(), error = %error, "directory opened but history could not be persisted");
            }
        }
        let snapshot = match &result {
            Ok(entries) => DirectorySnapshot {
                entries: entries.clone().into(),
                error: None,
            },
            Err(error) => DirectorySnapshot {
                entries: Vec::new().into(),
                error: Some(format!("{error:#}")),
            },
        };
        tracing::info!(directory = %relative.display(), entries = snapshot.entries.len(), elapsed_ms = started.elapsed().as_millis(), success = result.is_ok(), "project directory loaded");
        if let Err(error) = &result {
            tracing::error!(directory = %relative.display(), error = %error, "project directory load failed");
        }
        DATA_CONTEXT.project.set_directory(relative, snapshot);
        result.map(|_| ())
    }
}
