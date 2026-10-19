use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT, domain::settings::GeneralSettings, infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn load_core(&self) -> Result<()> {
        let _guard = self.io.lock().await;
        let value = INFRASTRUCTURE.settings.load().await?;
        INFRASTRUCTURE.proxy.validate(&value.lsp_proxy)?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        DATA_CONTEXT.settings.set(value);
        Ok(())
    }

    pub(super) async fn save_core(&self, mut value: GeneralSettings) -> Result<()> {
        let _guard = self.io.lock().await;
        value.lsp_proxy.url = value.lsp_proxy.url.trim().to_owned();
        INFRASTRUCTURE.proxy.validate(&value.lsp_proxy)?;
        INFRASTRUCTURE.settings.save(value.clone()).await?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        tracing::info!(auto_format = value.auto_format, "general settings saved");
        DATA_CONTEXT.settings.set(value);
        Ok(())
    }
}
