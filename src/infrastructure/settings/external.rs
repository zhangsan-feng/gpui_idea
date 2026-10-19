use super::{SettingsStore, core};
use crate::domain::settings::GeneralSettings;
use anyhow::Result;

impl SettingsStore {
    pub(crate) async fn load(&self) -> Result<GeneralSettings> {
        tokio::task::spawn_blocking(core::load).await?
    }

    pub(crate) async fn save(&self, value: GeneralSettings) -> Result<()> {
        tokio::task::spawn_blocking(move || core::save(value)).await?
    }
}
