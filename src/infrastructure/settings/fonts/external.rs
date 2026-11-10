use super::core;
use crate::{domain::settings::FontAsset, infrastructure::settings::SettingsStore};
use anyhow::Result;

impl SettingsStore {
    pub(crate) async fn load_fonts(&self) -> Result<Vec<FontAsset>> {
        tokio::task::spawn_blocking(core::load).await?
    }
}
