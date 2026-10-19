use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::language_server::{ManagedServer, ServerManagementAction},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn manage_servers_core(
        &self,
        action: ServerManagementAction,
    ) -> Result<Vec<ManagedServer>> {
        let _guard = self.server_io.lock().await;
        match INFRASTRUCTURE.language_servers.manage(action).await {
            Ok(servers) => {
                DATA_CONTEXT.settings.set_servers(servers.clone());
                Ok(servers)
            }
            Err(error) => {
                tracing::error!(error = %format!("{error:#}"), "LSP management operation failed");
                Err(error)
            }
        }
    }
}
