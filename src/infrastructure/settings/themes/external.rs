use super::{core, network};
use crate::{
    domain::theme::{ImportedTheme, OnlineTheme, ThemeSearch},
    infrastructure::settings::SettingsStore,
};
use anyhow::Result;
use std::path::PathBuf;

impl SettingsStore {
    pub(crate) async fn import_themes(&self, path: PathBuf) -> Result<Vec<ImportedTheme>> {
        tokio::task::spawn_blocking(move || core::import(&path)).await?
    }
    pub(crate) async fn search_themes(&self, query: String, offset: usize) -> Result<ThemeSearch> {
        tokio::task::spawn_blocking(move || network::search(query, offset)).await?
    }
    pub(crate) async fn download_themes(
        &self,
        extension: OnlineTheme,
    ) -> Result<Vec<ImportedTheme>> {
        tokio::task::spawn_blocking(move || network::download(extension)).await?
    }
}
