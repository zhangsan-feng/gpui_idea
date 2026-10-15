use super::{ProjectStore, core};
use anyhow::Result;
use std::path::PathBuf;
impl ProjectStore {
    pub(crate) async fn validate(&self, path: PathBuf) -> Result<PathBuf> {
        tokio::task::spawn_blocking(move || core::validate(&path)).await?
    }
    pub(crate) async fn create(&self, parent: PathBuf, name: String) -> Result<PathBuf> {
        tokio::task::spawn_blocking(move || core::create(&parent, &name)).await?
    }
    pub(crate) async fn launch(&self, path: PathBuf) -> Result<()> {
        tokio::task::spawn_blocking(move || core::launch(&path)).await?
    }
}
