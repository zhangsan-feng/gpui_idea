use super::SettingsService;
use crate::domain::settings::GeneralSettings;
use anyhow::Result;

impl SettingsService {
    pub(crate) fn managed_servers(&self) -> Vec<crate::domain::language_server::ManagedServer> {
        crate::data_context::DATA_CONTEXT.settings.servers()
    }

    pub(crate) async fn manage_servers(
        &self,
        action: crate::domain::language_server::ServerManagementAction,
    ) -> Result<Vec<crate::domain::language_server::ManagedServer>> {
        self.manage_servers_core(action).await
    }

    pub(crate) async fn load(&self) -> Result<()> {
        self.load_core().await
    }
    pub(crate) async fn save(&self, value: GeneralSettings) -> Result<()> {
        self.save_core(value).await
    }
}
