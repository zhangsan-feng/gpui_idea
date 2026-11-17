use super::DocumentService;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Context, Result};
use std::path::PathBuf;

impl DocumentService {
    pub(super) async fn last_modified_core(
        &self,
        path: PathBuf,
    ) -> Result<Option<std::time::SystemTime>> {
        let parent = path.parent().context("文件没有所在目录")?.to_owned();
        let name = path.file_name().context("文件没有名称")?.into();
        Ok(INFRASTRUCTURE
            .filesystem
            .metadata(parent, name)
            .await?
            .modified)
    }

    pub(super) async fn reveal_location_core(&self, path: PathBuf) -> Result<()> {
        let scope_root = DATA_CONTEXT
            .documents
            .snapshot(&path)
            .map(|document| document.scope_root.clone())
            .or_else(|| {
                if DATA_CONTEXT.has_project() && path.starts_with(DATA_CONTEXT.project_root()) {
                    Some(DATA_CONTEXT.project_root().to_owned())
                } else {
                    path.parent().map(ToOwned::to_owned)
                }
            })
            .context("文件没有所在目录")?;
        let relative = path
            .strip_prefix(&scope_root)
            .context("文件不在文档目录内")?
            .to_owned();
        INFRASTRUCTURE
            .filesystem
            .reveal_entry(scope_root, relative)
            .await
    }
}
