mod css;
mod go;
mod html;
mod java;
mod javascript;
mod json;
mod markdown;
mod php;
mod python;
mod rust;
mod toml;
mod yaml;
use super::managed::InstallContext;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub(super) struct Language {
    pub family: &'static str,
    pub key: &'static str,
    pub language: &'static str,
    pub name: &'static str,
    pub managed_name: &'static str,
    pub managed_languages: &'static str,
    pub markers: &'static [&'static str],
    pub sections: &'static [&'static str],
    pub prepare: fn(&Path, &InstallContext) -> Result<(Command, Option<String>)>,
    pub install: fn(&InstallContext) -> Result<()>,
    pub download: Option<Download>,
    pub configuration: fn(&str) -> Value,
    pub initialization_options: fn() -> Value,
    pub native_version: Option<&'static str>,
    pub detect_workspace: bool,
    pub workspace_markers: &'static [&'static str],
}

pub(super) const ALL: &[Language] = &[
    rust::LANGUAGE,
    go::LANGUAGE,
    python::LANGUAGE,
    php::LANGUAGE,
    javascript::LANGUAGE,
    java::LANGUAGE,
    toml::LANGUAGE,
    markdown::LANGUAGE,
    yaml::LANGUAGE,
    html::LANGUAGE,
    css::LANGUAGE,
    json::LANGUAGE,
];

pub(super) struct Download {
    pub latest: fn(&InstallContext) -> Result<String>,
    pub release: fn(&Path, &str, &InstallContext) -> Result<(String, PathBuf)>,
}

// Preserve the settings list and maintenance order independently of root detection.
const MANAGED: &[Language] = &[
    rust::LANGUAGE,
    go::LANGUAGE,
    java::LANGUAGE,
    toml::LANGUAGE,
    markdown::LANGUAGE,
    python::LANGUAGE,
    php::LANGUAGE,
    javascript::LANGUAGE,
    html::LANGUAGE,
    yaml::LANGUAGE,
];

mod core;
pub(super) use core::{
    configuration, download_release, get, initialization_options, latest_release, managed, settings,
};
