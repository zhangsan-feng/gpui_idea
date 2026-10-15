use super::{ProjectsService, core};
use anyhow::Result;
use std::path::PathBuf;
impl ProjectsService {
    pub(crate) async fn open(&self, path: PathBuf) -> Result<()> {
        core::open(path).await
    }
    pub(crate) async fn create(&self, parent: PathBuf, name: String) -> Result<PathBuf> {
        core::create(parent, name).await
    }
    pub(crate) async fn forget(&self, path: PathBuf) -> Result<()> {
        crate::application::APPLICATION
            .history
            .forget_project(path)
            .await
    }
}
