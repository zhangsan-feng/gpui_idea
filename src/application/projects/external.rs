use super::{ProjectsService, core};
use anyhow::Result;
use std::path::PathBuf;
impl ProjectsService {
    pub(crate) async fn open(&self, path: PathBuf) -> Result<()> {
        core::open(path).await
    }
    pub(crate) async fn create(
        &self,
        request: crate::domain::project_creation::CreateProject,
    ) -> Result<PathBuf> {
        core::create(request).await
    }
    pub(crate) async fn toolchain(
        &self,
        language: crate::domain::project_creation::ProjectLanguage,
        executable: String,
    ) -> Result<crate::domain::project_creation::ToolchainInfo> {
        crate::infrastructure::INFRASTRUCTURE
            .projects
            .toolchain(language, executable)
            .await
    }
    pub(crate) async fn forget(&self, path: PathBuf) -> Result<()> {
        crate::application::APPLICATION
            .history
            .forget_project(path)
            .await
    }
}
