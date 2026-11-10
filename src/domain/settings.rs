use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ContextMenuAction {
    Refresh,
    Install,
    Remove,
}

#[derive(Clone, Default, PartialEq, Eq, Deserialize)]
pub(crate) struct ContextMenuStatus {
    pub(crate) supported: bool,
    pub(crate) installed: bool,
    pub(crate) has_entries: bool,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GeneralSettings {
    pub(crate) appearance: AppearanceSettings,
    pub(crate) auto_format: bool,
    pub(crate) soft_wrap: bool,
    pub(crate) lsp_proxy: ProxySettings,
}

/// Published settings state. GUI drafts remain separate from this snapshot.
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct SettingsDataContextSnapshot {
    pub(crate) general: std::sync::Arc<GeneralSettings>,
    pub(crate) context_menu: Option<ContextMenuStatus>,
    pub(crate) servers: std::sync::Arc<[crate::domain::language_server::ManagedServer]>,
    pub(crate) fonts: std::sync::Arc<[FontAsset]>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FontAsset {
    pub(crate) id: String,
    pub(crate) families: Vec<String>,
    pub(crate) bytes: std::sync::Arc<[u8]>,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ColorTheme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppearanceSettings {
    #[serde(default)]
    pub(crate) font_defaults_version: u8,
    pub(crate) theme: ColorTheme,
    pub(crate) selected_theme: Option<String>,
    pub(crate) imported_themes: Vec<crate::domain::theme::ImportedTheme>,
    pub(crate) ui_font: String,
    pub(crate) ui_size: u16,
    pub(crate) editor_font: String,
    pub(crate) editor_size: u16,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            font_defaults_version: 2,
            theme: ColorTheme::Light,
            selected_theme: None,
            imported_themes: Vec::new(),
            ui_font: "JetBrains Mono".into(),
            ui_size: 13,
            editor_font: "JetBrains Mono".into(),
            editor_size: 13,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ProxySettings {
    pub(crate) enabled: bool,
    pub(crate) url: String,
}
