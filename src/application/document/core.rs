use super::{DocumentService, OpenedDocument};
use crate::application::APPLICATION;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};

use anyhow::{Result, bail};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

impl DocumentService {
    pub(super) async fn save_dirty_core(&self) -> Result<()> {
        loop {
            if DATA_CONTEXT.documents.autosave_paused() {
                return Ok(());
            }
            for document in DATA_CONTEXT.documents.dirty_documents() {
                if document.saving || document.save_error.is_some() {
                    continue;
                }
                if let Err(error) = self.save_core(document.path.clone(), true).await {
                    tracing::error!(path = %document.path.display(), %error, "automatic document save failed");
                }
            }
            if DATA_CONTEXT.documents.autosave_paused()
                || !DATA_CONTEXT
                    .documents
                    .dirty_documents()
                    .iter()
                    .any(|doc| doc.save_error.is_none())
            {
                return Ok(());
            }
            // A slow save can finish after the next edit's timer fired; retry new revisions.
            INFRASTRUCTURE
                .timer
                .delay("document-autosave-retry", std::time::Duration::from_secs(3))
                .await?;
        }
    }

    pub(super) fn release_clean_core(&self, path: &Path) -> bool {
        DATA_CONTEXT.documents.release_clean(path)
    }

    pub(super) fn edit_core(&self, path: &Path, text: ropey::Rope, changed: bool) -> Result<()> {
        DATA_CONTEXT.documents.edit(path, text, changed)
    }

    pub(super) fn attach_editor_buffer_core(
        &self,
        path: &Path,
        text: ropey::Rope,
        revision: u64,
    ) -> Result<()> {
        DATA_CONTEXT.documents.attach_buffer(path, text, revision)
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
            crate::application::lifecycle::warm_language_server_document(snapshot.clone());
            return Ok(OpenedDocument {
                snapshot,
                initial_text,
            });
        }
        let loaded = INFRASTRUCTURE.filesystem.read_document(root, path).await?;
        let snapshot = DATA_CONTEXT.documents.insert(loaded.document);
        crate::application::lifecycle::warm_language_server_document(snapshot.clone());
        Ok(OpenedDocument {
            snapshot,
            initial_text: loaded.initial_text,
        })
    }
    pub(super) async fn save_core(&self, path: PathBuf, automatic: bool) -> Result<()> {
        let formatted = self.format_before_save(&path, automatic).await;
        let _refresh = APPLICATION.file_watch.lock_refresh().await;
        let current = DATA_CONTEXT
            .documents
            .snapshot(&path)
            .ok_or_else(|| anyhow::anyhow!("文件尚未打开"))?;
        let relative = path.strip_prefix(&current.scope_root)?.to_owned();
        let change = INFRASTRUCTURE
            .filesystem
            .begin_gui_file_change(current.scope_root.clone(), vec![relative])
            .await?;
        let Some(snapshot) = DATA_CONTEXT
            .documents
            .begin_save(&path, automatic, formatted)?
        else {
            change.finish(Ok(()))?;
            return Ok(());
        };
        let revision = snapshot.revision;
        let started = std::time::Instant::now();
        let result = change.finish(
            crate::event_bus::EVENT_BUS
                .timer_event
                .save_file(Arc::clone(&snapshot))
                .await,
        );
        let error = result.as_ref().err().map(|error| format!("{error:#}"));
        DATA_CONTEXT
            .documents
            .finish_save(&path, revision, result)?;
        tracing::info!(file = %path.display(), revision, elapsed_ms = started.elapsed().as_millis(), success = error.is_none(), "document save completed");
        if let Some(error) = error {
            bail!(error);
        }
        // Local native events are excluded; update already loaded statistics here.
        if !DATA_CONTEXT.project.statistics().scanned_at.is_empty() {
            if let Err(error) = APPLICATION.project.statistics().await {
                tracing::warn!(%error, "document saved but project statistics refresh failed");
            }
        }
        Ok(())
    }
}
