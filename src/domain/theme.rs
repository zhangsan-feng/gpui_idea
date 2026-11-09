use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ThemeToken {
    pub(crate) foreground: Option<String>,
    pub(crate) font_style: Option<String>,
}

/// Self-contained imported palette; no dependency on the source file after import.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ImportedTheme {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) source: String,
    #[serde(default)]
    pub(crate) package_name: String,
    pub(crate) dark: bool,
    pub(crate) colors: BTreeMap<String, String>,
    pub(crate) syntax: BTreeMap<String, ThemeToken>,
    pub(crate) notes: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct OnlineTheme {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) version: String,
    pub(crate) downloads: u64,
    pub(crate) download_url: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ThemeSearch {
    pub(crate) extensions: Vec<OnlineTheme>,
    pub(crate) next_offset: Option<usize>,
}

pub(crate) struct InstalledThemePackage<'a> {
    pub(crate) source: &'a str,
    pub(crate) name: &'a str,
    pub(crate) themes: Vec<&'a ImportedTheme>,
}
