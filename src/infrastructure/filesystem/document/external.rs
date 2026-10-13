use super::{LoadedFile, core};
use crate::domain::document::DocumentSnapshot;
use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};

pub async fn read(root: PathBuf, path: PathBuf) -> Result<LoadedFile> {
    tokio::task::spawn_blocking(move || core::read(&root, &path))
        .await
        .context("读取任务中止")?
}
pub async fn save(root: PathBuf, document: Arc<DocumentSnapshot>) -> Result<Arc<[u8]>> {
    tokio::task::spawn_blocking(move || core::save(&root, &document))
        .await
        .context("保存任务中止")?
}
