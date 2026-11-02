use super::DocumentService;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Context, Result};
use std::path::PathBuf;

impl DocumentService {
    pub(super) async fn reveal_location_core(&self, path: PathBuf) -> Result<()> {
        let document = DATA_CONTEXT
            .documents
            .snapshot(&path)
            .context("文档尚未打开")?;
        let relative = path
            .strip_prefix(&document.scope_root)
            .context("文件不在文档目录内")?
            .to_owned();
        INFRASTRUCTURE
            .filesystem
            .reveal_entry(document.scope_root.clone(), relative)
            .await
    }
}
