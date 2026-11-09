use super::SettingsService;
use crate::domain::settings::GeneralSettings;
use anyhow::Result;

impl SettingsService {
    pub(crate) fn installed_theme_packages<'a>(
        &self,
        settings: &'a GeneralSettings,
    ) -> Vec<crate::domain::theme::InstalledThemePackage<'a>> {
        self.installed_theme_packages_core(settings)
    }

    pub(crate) fn select_theme(&self, settings: &mut GeneralSettings, id: &str) -> bool {
        self.select_theme_core(settings, id)
    }

    pub(crate) fn select_default_theme(&self, settings: &mut GeneralSettings) {
        self.select_default_theme_core(settings);
    }

    pub(crate) fn uninstall_theme(&self, settings: &mut GeneralSettings, source: &str) -> bool {
        self.uninstall_theme_core(settings, source)
    }

    pub(crate) fn stage_themes(
        &self,
        settings: &mut GeneralSettings,
        themes: Vec<crate::domain::theme::ImportedTheme>,
    ) {
        self.stage_themes_core(settings, themes);
    }

    pub(crate) async fn import_themes(
        &self,
        path: std::path::PathBuf,
    ) -> Result<Vec<crate::domain::theme::ImportedTheme>> {
        self.import_themes_core(path).await
    }

    pub(crate) async fn search_themes(
        &self,
        query: String,
        offset: usize,
    ) -> Result<crate::domain::theme::ThemeSearch> {
        self.search_themes_core(query, offset).await
    }

    pub(crate) async fn download_themes(
        &self,
        extension: crate::domain::theme::OnlineTheme,
    ) -> Result<Vec<crate::domain::theme::ImportedTheme>> {
        self.download_themes_core(extension).await
    }

    pub(crate) async fn context_menu(
        &self,
        action: crate::domain::settings::ContextMenuAction,
    ) -> Result<crate::domain::settings::ContextMenuStatus> {
        self.context_menu_core(action).await
    }

    pub(crate) fn managed_servers(&self) -> Vec<crate::domain::language_server::ManagedServer> {
        crate::data_context::DATA_CONTEXT
            .settings
            .snapshot()
            .servers
            .to_vec()
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
