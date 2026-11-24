use super::{SettingsStore, core};
use crate::domain::settings::GeneralSettings;
use anyhow::Result;

impl SettingsStore {
    pub(crate) async fn load(&self) -> Result<GeneralSettings> {
        tokio::task::spawn_blocking(core::load).await?
    }

    pub(crate) async fn save(
        &self,
        value: GeneralSettings,
        preserve_editor_size: bool,
    ) -> Result<GeneralSettings> {
        tokio::task::spawn_blocking(move || core::save(value, preserve_editor_size)).await?
    }

    pub(crate) async fn save_editor_font_size(&self, size: u16) -> Result<()> {
        tokio::task::spawn_blocking(move || core::save_editor_font_size(size)).await?
    }
}
