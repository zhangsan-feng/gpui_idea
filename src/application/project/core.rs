mod batch;
mod content_search;
mod file_tree;
mod run;
mod run_configurations;
mod run_sessions;

use super::ProjectService;
use crate::application::APPLICATION;
use crate::domain::project::DirectorySnapshot;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail};
use std::{path::PathBuf, time::Instant};

impl ProjectService {
    pub(super) async fn statistics_core(&self) -> Result<()> {
        let _permit = self.reads.acquire().await?;
        let result = INFRASTRUCTURE
            .filesystem
            .statistics(DATA_CONTEXT.project_root().to_owned())
            .await;
        match result {
            Ok(snapshot) => {
                DATA_CONTEXT.project.set_statistics(snapshot);
                Ok(())
            }
            Err(error) => {
                DATA_CONTEXT.project.set_statistics(
                    crate::domain::statistics::StatisticsSnapshot {
                        error: Some(format!("{error:#}")),
                        ..Default::default()
                    },
                );
                Err(error)
            }
        }
    }
    pub(super) async fn git_status_core(&self) -> Result<()> {
        let result = INFRASTRUCTURE
            .git
            .snapshot(DATA_CONTEXT.project_root().to_owned())
            .await;
        match result {
            Ok(status) => {
                DATA_CONTEXT.project.set_git(status);
                Ok(())
            }
            Err(error) => {
                DATA_CONTEXT
                    .project
                    .set_git(crate::domain::git::GitSnapshot {
                        error: Some(format!("{error:#}")),
                        ..Default::default()
                    });
                Err(error)
            }
        }
    }
    pub(super) async fn git_details_core(&self, hash: String) -> Result<()> {
        let result = INFRASTRUCTURE
            .git
            .details(DATA_CONTEXT.project_root().to_owned(), hash.clone())
            .await;
        match result {
            Ok(details) => {
                DATA_CONTEXT.project.set_git_details(details);
                Ok(())
            }
            Err(error) => {
                DATA_CONTEXT
                    .project
                    .set_git_details(crate::domain::git::GitDetails {
                        hash,
                        error: Some(format!("{error:#}")),
                        ..Default::default()
                    });
                Err(error)
            }
        }
    }
    // The caller holds file_operations and the file-watch refresh lock.
    pub(in crate::application::project) async fn move_unlocked(
        &self,
        source: PathBuf,
        target_directory: PathBuf,
    ) -> Result<PathBuf> {
        file_tree::ensure_idle(&source)?;
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
        let change = INFRASTRUCTURE
            .filesystem
            .begin_gui_file_change(
                DATA_CONTEXT.project_root().to_owned(),
                vec![source.clone(), destination.clone()],
            )
            .await?;
        change.finish(
            INFRASTRUCTURE
                .filesystem
                .rename(
                    DATA_CONTEXT.project_root().to_owned(),
                    source.clone(),
                    destination.clone(),
                )
                .await,
        )?;
        self.invalidate_moved_run_target(&source, &destination);
        self.refresh_local_directories(
            &[
                source.parent().unwrap_or(std::path::Path::new("")),
                &target_directory,
            ],
            Some((&source, &destination)),
        )
        .await;
        tracing::info!(destination = %destination.display(), "project entry moved");
        Ok(destination)
    }

    pub(super) async fn load_core(&self, relative: PathBuf) -> Result<()> {
        let _refresh = APPLICATION.file_watch.lock_refresh().await;
        let _permit = self.reads.acquire().await?;
        let started = Instant::now();
        let result = INFRASTRUCTURE
            .filesystem
            .list_directory(DATA_CONTEXT.project_root().to_owned(), relative.clone())
            .await;
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
