use super::{Git, core};
use crate::domain::git::{GitDataContextSnapshot, GitDetailsDataContextSnapshot};
use anyhow::Result;
use std::path::PathBuf;

impl Git {
    pub(crate) async fn create(
        &self,
        root: PathBuf,
        request: crate::domain::git::GitCreateRequest,
    ) -> Result<String> {
        super::create::create(root, request).await
    }
    pub(crate) async fn operate(
        &self,
        root: PathBuf,
        action: crate::domain::git::GitAction,
    ) -> Result<String> {
        super::operations::operate(root, action).await
    }

    pub(crate) async fn diff(
        &self,
        root: PathBuf,
        path: PathBuf,
        staged: bool,
        commit: Option<String>,
        original_path: Option<PathBuf>,
    ) -> Result<String> {
        super::operations::diff(root, path, staged, commit, original_path).await
    }
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
