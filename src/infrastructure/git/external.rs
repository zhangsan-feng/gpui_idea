use super::{Git, core};
use crate::domain::git::{GitDataContextSnapshot, GitDetailsDataContextSnapshot};
use anyhow::Result;
use std::path::PathBuf;

impl Git {
    pub(crate) async fn snapshot(&self, root: PathBuf) -> Result<GitDataContextSnapshot> {
        core::snapshot(root).await
    }

    pub(crate) async fn details(
        &self,
        root: PathBuf,
        hash: String,
    ) -> Result<GitDetailsDataContextSnapshot> {
        core::details(root, hash).await
    }
}
