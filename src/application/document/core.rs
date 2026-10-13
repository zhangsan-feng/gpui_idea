use super::{DocumentService, OpenedDocument};
use crate::application::APPLICATION;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};

use anyhow::{Result, bail};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

impl DocumentService {
    pub(super) fn release_clean_core(&self, path: &Path) -> bool {
        DATA_CONTEXT.documents.release_clean(path)
    }

    pub(super) fn edit_core(&self, path: &Path, text: ropey::Rope, changed: bool) -> Result<()> {
        DATA_CONTEXT.documents.edit(path, text, changed)
    }

    pub(super) async fn record_opened(&self, opened: &OpenedDocument) {
        if let Err(error) = APPLICATION
            .history
            .record_opened(
                crate::domain::history::OpenKind::File,
                opened.snapshot.path.clone(),
            )
            .await
        {
            tracing::warn!(file = %opened.snapshot.path.display(), error = %error, "file opened but history could not be persisted");
        }
    }
    pub(super) fn resolve(&self, relative: &Path) -> Result<PathBuf> {
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
        {
            bail!("无效的项目文件路径");
        }
        Ok(DATA_CONTEXT.project_root().join(relative))
    }
    pub(super) async fn open_core(&self, relative: PathBuf) -> Result<OpenedDocument> {
        let path = self.resolve(&relative)?;
        self.open_at_root(DATA_CONTEXT.project_root().to_owned(), path)
            .await
    }
    pub(super) async fn open_external_core(&self, path: PathBuf) -> Result<OpenedDocument> {
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            bail!("拖入的文件必须具有有效的绝对路径");
        }
        let root = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("文件没有父目录"))?
            .to_owned();
        tracing::info!(file = %path.display(), "standalone file open requested by external drop");
        self.open_at_root(root, path).await
    }
    async fn open_at_root(&self, root: PathBuf, path: PathBuf) -> Result<OpenedDocument> {
        if let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&path) {
            let buffer = snapshot.text.clone();
            let initial_text = tokio::task::spawn_blocking(move || buffer.to_string()).await?;
            return Ok(OpenedDocument {
                snapshot,
                initial_text,
            });
        }
        let loaded = INFRASTRUCTURE.filesystem.read_document(root, path).await?;
        let snapshot = DATA_CONTEXT.documents.insert(loaded.document);
        Ok(OpenedDocument {
            snapshot,
            initial_text: loaded.initial_text,
        })
    }
    pub(super) async fn save_core(&self, path: PathBuf) -> Result<()> {
        let snapshot = DATA_CONTEXT.documents.begin_save(&path)?;
        let revision = snapshot.revision;
        let started = std::time::Instant::now();
        let result = INFRASTRUCTURE
            .filesystem
            .save_document(snapshot.scope_root.clone(), Arc::clone(&snapshot))
            .await;
        let error = result.as_ref().err().map(|error| format!("{error:#}"));
        DATA_CONTEXT
            .documents
            .finish_save(&path, revision, result)?;
        tracing::info!(file = %path.display(), revision, elapsed_ms = started.elapsed().as_millis(), success = error.is_none(), "document save completed");
        if let Some(error) = error {
            bail!(error);
        }
        Ok(())
    }
}
