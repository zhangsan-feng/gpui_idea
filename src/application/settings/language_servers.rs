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
        // Different families install independently; cleanup requires exclusive cache access.
        let cleanup = matches!(&action, ServerManagementAction::CleanHistory);
        let _exclusive = if cleanup {
            Some(self.server_io.write().await)
        } else {
            None
        };
        let _shared = if cleanup {
            None
        } else {
            Some(self.server_io.read().await)
        };
        let key = match &action {
            ServerManagementAction::Update(key) | ServerManagementAction::Select { key, .. } => {
                key.as_str()
            }
            ServerManagementAction::Refresh => "refresh",
            ServerManagementAction::CleanHistory => "cleanup",
        }
        .to_owned();
        tracing::info!(key, "LSP management operation started");
        let result = INFRASTRUCTURE.language_servers.manage(action).await;
        if let Err(error) = &result {
            tracing::error!(key, error = %format!("{error:#}"), "LSP management operation failed");
        }
        // Read and publish together so overlapping completions cannot restore an older list.
        // Also refresh after failure: cleanup may have completed only part of its work.
        let _snapshot = self.server_snapshot_io.lock().await;
        let servers = INFRASTRUCTURE.language_servers.managed_servers().await;
        if let Ok(servers) = &servers {
            DATA_CONTEXT.settings.set_servers(servers.clone());
        }
        result?;
        tracing::info!(key, "LSP management operation completed");
        servers
    }
}
