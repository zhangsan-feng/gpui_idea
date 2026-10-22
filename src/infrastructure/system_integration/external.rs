use super::{SystemIntegration, core};
use crate::domain::settings::{ContextMenuAction, ContextMenuStatus};
use anyhow::Result;

impl SystemIntegration {
    pub(crate) async fn context_menu(
        &self,
        action: ContextMenuAction,
    ) -> Result<ContextMenuStatus> {
        tokio::task::spawn_blocking(move || core::context_menu(action)).await?
    }
}
