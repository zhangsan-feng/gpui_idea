use super::DocumentService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::image_preview::{ImagePreviewSnapshot, PreviewImageFormat},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Context as _, Result, ensure};
use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

impl DocumentService {
    pub(super) async fn open_image_core(&self, path: PathBuf) -> Result<Arc<ImagePreviewSnapshot>> {
        let path = if path.is_absolute() {
            path
        } else {
            self.resolve(&path)?
        };
        let refresh = crate::application::APPLICATION
            .file_watch
            .lock_refresh()
            .await;
        if let Some(snapshot) = DATA_CONTEXT.images.snapshot(&path) {
            return Ok(snapshot);
        }
        let format = PreviewImageFormat::for_path(&path).context("不支持的图片格式")?;
        let bytes = self.read_image_bytes(&path).await?;
        let snapshot = DATA_CONTEXT
            .images
            .store(path.clone(), format, bytes.into(), None);
        drop(refresh);
        tracing::info!(path = %path.display(), bytes = snapshot.bytes.len(), "image preview loaded");
        Ok(snapshot)
    }

    pub(super) async fn refresh_image_core(&self, expected: Arc<ImagePreviewSnapshot>) {
        let (bytes, error) = match self.read_image_bytes(&expected.path).await {
            Ok(bytes) => (Arc::from(bytes), None),
            Err(error) => {
                tracing::warn!(path = %expected.path.display(), %error, "image preview reload failed");
                (
                    Arc::from([]),
                    Some(format!("图片已删除或无法读取：{error:#}")),
                )
            }
        };
        DATA_CONTEXT
            .images
            .replace_if_current(&expected, bytes, error);
    }

    async fn read_image_bytes(&self, path: &Path) -> Result<Vec<u8>> {
        ensure!(
            path.is_absolute()
                && !path
                    .components()
                    .any(|part| matches!(part, Component::ParentDir)),
            "无效的图片路径"
        );
        let root = if DATA_CONTEXT.has_project() && path.starts_with(DATA_CONTEXT.project_root()) {
            DATA_CONTEXT.project_root()
        } else {
            path.parent().context("图片没有父目录")?
        };
        // Reuse the sandboxed, size-bounded reader; actual IO uses spawn_blocking.
        INFRASTRUCTURE
            .filesystem
            .read_file(
                root.to_owned(),
                path.strip_prefix(root)?.to_owned(),
                32 * 1024 * 1024,
            )
            .await
    }
}
