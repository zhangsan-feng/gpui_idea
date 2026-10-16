use super::{Git, core};
use crate::domain::git::{GitDetails, GitSnapshot};
use anyhow::Result;
use std::path::PathBuf;

impl Git {
    pub(crate) async fn snapshot(&self, root: PathBuf) -> Result<GitSnapshot> {
        core::snapshot(root).await
    }

    pub(crate) async fn details(&self, root: PathBuf, hash: String) -> Result<GitDetails> {
        core::details(root, hash).await
    }
}
