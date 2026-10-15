use crate::{application::APPLICATION, domain::history::OpenKind, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail};
use std::path::PathBuf;
pub(super) async fn open(path: PathBuf) -> Result<()> {
    let root = INFRASTRUCTURE.projects.validate(path).await?;
    let current = crate::data_context::DATA_CONTEXT.project_root();
    if root
        .to_string_lossy()
        .trim_start_matches("\\\\?\\")
        .eq_ignore_ascii_case(current.to_string_lossy().trim_start_matches("\\\\?\\"))
    {
        bail!("项目已在当前工作区打开");
    }
    INFRASTRUCTURE.projects.launch(root.clone()).await?;
    if let Err(error) = APPLICATION
        .history
        .record_opened(OpenKind::Project, root)
        .await
    {
        tracing::warn!(%error, "project launched but recent history could not be saved");
    }
    Ok(())
}
pub(super) async fn create(parent: PathBuf, name: String) -> Result<PathBuf> {
    let name = APPLICATION.project.validate_file_name(&name)?;
    let root = INFRASTRUCTURE.projects.create(parent, name).await?;
    APPLICATION
        .history
        .record_opened(OpenKind::Project, root.clone())
        .await?;
    Ok(root)
}
