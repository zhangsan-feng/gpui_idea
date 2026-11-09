use super::SettingsService;
use crate::{
    domain::theme::{ImportedTheme, OnlineTheme, ThemeSearch},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;
use std::path::PathBuf;

impl SettingsService {
    pub(super) fn installed_theme_packages_core<'a>(
        &self,
        settings: &'a crate::domain::settings::GeneralSettings,
    ) -> Vec<crate::domain::theme::InstalledThemePackage<'a>> {
        let mut packages = std::collections::BTreeMap::new();
        for theme in &settings.appearance.imported_themes {
            let package = packages.entry(theme.source.as_str()).or_insert_with(|| {
                crate::domain::theme::InstalledThemePackage {
                    source: theme.source.as_str(),
                    name: if theme.package_name.is_empty() {
                        theme.name.as_str()
                    } else {
                        theme.package_name.as_str()
                    },
                    themes: Vec::new(),
                }
            });
            package.themes.push(theme);
        }
        packages.into_values().collect()
    }

    pub(super) fn select_theme_core(
        &self,
        settings: &mut crate::domain::settings::GeneralSettings,
        id: &str,
    ) -> bool {
        let Some(theme) = settings
            .appearance
            .imported_themes
            .iter()
            .find(|theme| theme.id == id)
        else {
            return false;
        };
        settings.appearance.theme = if theme.dark {
            crate::domain::settings::ColorTheme::Dark
        } else {
            crate::domain::settings::ColorTheme::Light
        };
        settings.appearance.selected_theme = Some(theme.id.clone());
        true
    }

    pub(super) fn select_default_theme_core(
        &self,
        settings: &mut crate::domain::settings::GeneralSettings,
    ) {
        settings.appearance.selected_theme = None;
        settings.appearance.theme = Default::default();
    }

    pub(super) fn uninstall_theme_core(
        &self,
        settings: &mut crate::domain::settings::GeneralSettings,
        source: &str,
    ) -> bool {
        let before = settings.appearance.imported_themes.len();
        let restore_default = settings.appearance.imported_themes.iter().any(|theme| {
            theme.source == source
                && settings.appearance.selected_theme.as_deref() == Some(theme.id.as_str())
        });
        settings
            .appearance
            .imported_themes
            .retain(|theme| theme.source != source);
        let removed = settings.appearance.imported_themes.len() != before;
        if restore_default {
            self.select_default_theme_core(settings);
        }
        tracing::info!(
            source,
            removed,
            restore_default,
            "theme package removal staged"
        );
        removed
    }

    pub(super) fn stage_themes_core(
        &self,
        settings: &mut crate::domain::settings::GeneralSettings,
        themes: Vec<ImportedTheme>,
    ) {
        for theme in themes {
            if let Some(previous) = settings
                .appearance
                .imported_themes
                .iter_mut()
                .find(|previous| previous.id == theme.id)
            {
                *previous = theme;
            } else {
                settings.appearance.imported_themes.push(theme);
            }
        }
        if let Some(theme) = settings.appearance.selected_theme.as_ref().and_then(|id| {
            settings
                .appearance
                .imported_themes
                .iter()
                .find(|theme| &theme.id == id)
        }) {
            settings.appearance.theme = if theme.dark {
                crate::domain::settings::ColorTheme::Dark
            } else {
                crate::domain::settings::ColorTheme::Light
            };
        }
    }

    pub(super) async fn import_themes_core(&self, path: PathBuf) -> Result<Vec<ImportedTheme>> {
        let result = INFRASTRUCTURE.settings.import_themes(path.clone()).await;
        match &result {
            Ok(themes) => {
                tracing::info!(path = %path.display(), count = themes.len(), "VS Code themes imported")
            }
            Err(error) => tracing::warn!(path = %path.display(), %error, "theme import failed"),
        }
        result
    }

    pub(super) async fn search_themes_core(
        &self,
        query: String,
        offset: usize,
    ) -> Result<ThemeSearch> {
        tracing::debug!(%query, offset, "searching Open VSX themes");
        INFRASTRUCTURE.settings.search_themes(query, offset).await
    }

    pub(super) async fn download_themes_core(
        &self,
        extension: OnlineTheme,
    ) -> Result<Vec<ImportedTheme>> {
        tracing::info!(extension = %extension.id, version = %extension.version, "downloading Open VSX theme package");
        let result = INFRASTRUCTURE.settings.download_themes(extension).await;
        if let Err(error) = &result {
            tracing::warn!(%error, "theme package download failed");
        }
        result
    }
}
