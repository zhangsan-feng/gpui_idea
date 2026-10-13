use crate::domain::project::ProjectEntry;
use anyhow::Result;
use std::path::PathBuf;

pub async fn read(root: PathBuf, relative: PathBuf) -> Result<Vec<ProjectEntry>> {
    tokio::task::spawn_blocking(move || super::core::read(&root, &relative)).await?
}
