use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ContextMenuAction {
    Refresh,
    Install,
    Remove,
}

#[derive(Clone, Default, Deserialize)]
pub(crate) struct ContextMenuStatus {
    pub(crate) supported: bool,
    pub(crate) installed: bool,
    pub(crate) has_entries: bool,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GeneralSettings {
    pub(crate) auto_format: bool,
    pub(crate) lsp_proxy: ProxySettings,
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ProxySettings {
    pub(crate) enabled: bool,
    pub(crate) url: String,
}
