use super::LanguageServers;
use crate::domain::language_server::{ManagedServer, ServerManagementAction};
use anyhow::Result;

impl LanguageServers {
    pub(crate) async fn manage(&self, action: ServerManagementAction) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            use super::managed::management;
            match action {
                ServerManagementAction::Refresh => {}
                ServerManagementAction::CleanHistory => management::clean_history()?,
                ServerManagementAction::Select { key, directory } => {
                    management::select(&key, directory)?
                }
                ServerManagementAction::Update(key) => {
                    tracing::info!(key, "Checking managed LSP release from settings");
                    management::install_latest(&key, &management::install_context(None)?)?;
                }
            }
            Ok(())
        })
        .await?
    }

    pub(crate) async fn managed_servers(&self) -> Result<Vec<ManagedServer>> {
        tokio::task::spawn_blocking(super::managed::management::snapshot).await?
    }
}
