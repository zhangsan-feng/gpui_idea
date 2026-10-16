use super::{ProjectStore, core};
use anyhow::Result;
use std::path::PathBuf;
impl ProjectStore {
    pub(crate) async fn validate(&self, path: PathBuf) -> Result<PathBuf> {
        tokio::task::spawn_blocking(move || core::validate(&path)).await?
    }
    pub(crate) async fn create(
        &self,
        request: crate::domain::project_creation::CreateProject,
        name: String,
        environment: Vec<(String, String)>,
    ) -> Result<PathBuf> {
        super::scaffold::create(request, name, environment).await
    }
    pub(crate) async fn toolchain(
        &self,
        language: crate::domain::project_creation::ProjectLanguage,
        executable: String,
    ) -> Result<crate::domain::project_creation::ToolchainInfo> {
        super::toolchain::probe(language, executable).await
    }
    pub(crate) async fn launch(&self, path: PathBuf) -> Result<()> {
        tokio::task::spawn_blocking(move || core::launch(&path)).await?
    }
}
