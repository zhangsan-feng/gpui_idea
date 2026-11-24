use serde::{Deserialize, Serialize};

pub(crate) const EDITOR_FONT_SIZE_RANGE: std::ops::RangeInclusive<u16> = 9..=36;

#[derive(Clone, Debug)]
pub(crate) enum FileAssociationAction {
    Refresh,
    Configure(Vec<String>),
    Register(Vec<String>),
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct FileAssociationStatus {
    pub(crate) registered: bool,
    pub(crate) extensions: Vec<String>,
    pub(crate) default_extensions: Vec<String>,
    pub(crate) requested_settings_page: Option<DefaultAppsPage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DefaultAppsPage {
    Application,
    Overview,
}

pub(crate) const EDITOR_FILE_EXTENSIONS: &[&str] = &[
    ".txt", ".py", ".go", ".js", ".jsx", ".ts", ".tsx", ".rs", ".json", ".toml", ".md", ".yaml",
    ".yml", ".html", ".css", ".java", ".php", ".xml", ".sh", ".sql",
];

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GeneralSettings {
    pub(crate) appearance: AppearanceSettings,
    pub(crate) auto_format: bool,
    pub(crate) soft_wrap: bool,
    pub(crate) lsp_proxy: ProxySettings,
    #[serde(default = "default_ignored_directories")]
    pub(crate) ignored_directories: Vec<String>,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            appearance: Default::default(),
            auto_format: false,
            soft_wrap: false,
            lsp_proxy: Default::default(),
            ignored_directories: default_ignored_directories(),
        }
    }
}

pub(crate) fn default_ignored_directories() -> Vec<String> {
    [
        ".git",
        ".idea",
        "target",
        "node_modules",
        "dist",
        "build",
        "artifacts",
        ".codex",
        ".agents",
        ".venv",
        "__pycache__",
        ".gpui-run",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// Published settings state. GUI drafts remain separate from this snapshot.
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct SettingsDataContextSnapshot {
    pub(crate) general: std::sync::Arc<GeneralSettings>,
    pub(crate) context_menu: Option<ContextMenuStatus>,
    pub(crate) file_associations: Option<FileAssociationStatus>,
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
