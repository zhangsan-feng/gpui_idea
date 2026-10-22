use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::settings::{ContextMenuAction, ContextMenuStatus},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn context_menu_core(
        &self,
        action: ContextMenuAction,
    ) -> Result<ContextMenuStatus> {
        let _guard = self.system_io.lock().await;
        match INFRASTRUCTURE.system_integration.context_menu(action).await {
            Ok(status) => {
                DATA_CONTEXT.settings.set_context_menu(Some(status.clone()));
                Ok(status)
            }
            Err(error) => {
                DATA_CONTEXT.settings.set_context_menu(None);
                tracing::error!(?action, error = %format!("{error:#}"), "system integration failed");
                Err(error)
            }
        }
    }
}
