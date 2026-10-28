use super::{ALL, Download, Language, MANAGED};
use crate::infrastructure::language_server::managed::InstallContext;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(in crate::infrastructure::language_server) fn get(family: &str) -> Result<&'static Language> {
    ALL.iter()
        .find(|language| language.family == family)
        .with_context(|| format!("不支持的语言服务：{family}"))
}

pub(in crate::infrastructure::language_server) fn managed()
-> impl Iterator<Item = &'static Language> {
    MANAGED.iter()
}

pub(in crate::infrastructure::language_server) fn configuration(section: &str) -> Value {
    if section.is_empty() {
        return settings();
    }
    ALL.iter()
        .find(|language| language.sections.contains(&section))
        .map(|language| (language.configuration)(section))
        .unwrap_or(Value::Null)
}

pub(in crate::infrastructure::language_server) fn initialization_options(family: &str) -> Value {
    get(family)
        .map(|language| (language.initialization_options)())
        .unwrap_or_else(|_| json!({}))
}

pub(in crate::infrastructure::language_server) fn settings() -> Value {
    let mut settings = serde_json::Map::new();
    for language in ALL {
        for section in language.sections {
            // Rust settings are supplied as initialization options; nested sections
            // are served only in response to workspace/configuration requests.
            if *section != "rust-analyzer" && !section.contains('.') {
                settings.insert((*section).into(), (language.configuration)(section));
            }
        }
    }
    Value::Object(settings)
}

fn download(server: &str) -> Result<&'static Download> {
    managed()
        .find(|language| language.key == server)
        .and_then(|language| language.download.as_ref())
        .with_context(|| format!("未知服务器：{server}"))
}

pub(in crate::infrastructure::language_server) fn latest_release(
    server: &str,
    context: &InstallContext,
) -> Result<String> {
    (download(server)?.latest)(context)
}

pub(in crate::infrastructure::language_server) fn download_release(
    server: &str,
    stage: &Path,
    version: &str,
    context: &InstallContext,
) -> Result<(String, PathBuf)> {
    (download(server)?.release)(stage, version, context)
}
