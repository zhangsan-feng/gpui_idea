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
    pub initialization_options: fn(&Path) -> Value,
    pub native_version: Option<&'static str>,
    pub detect_workspace: fn(&Path) -> bool,
    pub file_run: Option<FileRunSupport>,
    pub file_extensions: &'static [&'static str],
    pub code_package: Option<CodePackageSupport>,
}

pub(super) struct CodePackageSupport {
    pub detect: fn(&[PathBuf]) -> bool,
    pub markers: &'static [&'static str],
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
mod package;
pub(super) use core::{
    configuration, download_release, get, initialization_options, latest_release, managed, settings,
};

pub(super) struct FileRunSupport {
    pub accepts_file: fn(&Path) -> bool,
    pub resolve:
        fn(&Path, &Path) -> Result<crate::domain::run::RunConfigurationDataContextSnapshot>,
    pub server: Option<ServerRunSupport>,
}

pub(super) struct ServerRunSupport {
    pub method: &'static str,
    pub capability: &'static str,
    pub parameters: fn(&str) -> Value,
    pub parse: fn(
        &Path,
        &Value,
    ) -> Result<Option<crate::domain::run::RunConfigurationDataContextSnapshot>>,
}
