use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use managed::http;
use serde_json::{Value, json};
use std::path::Path;
use std::path::PathBuf;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    Value::Null
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    json!({})
}

pub(in crate::infrastructure::language_server) fn prepare(
    _root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    let mut command = Command::new(managed::download_server("marksman", context)?);
    command.arg("server");
    Ok((command, None))
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::download_server("marksman", context).map(|_| ())
}

pub(in crate::infrastructure::language_server) fn latest_release(
    _context: &InstallContext,
) -> Result<String> {
    http::latest("artempyanykh/marksman")
}
pub(in crate::infrastructure::language_server) fn download_release(
    stage: &Path,
    version: &str,
    context: &InstallContext,
) -> Result<(String, PathBuf)> {
    anyhow::ensure!(
        std::env::consts::ARCH == "x86_64",
        "Marksman Windows 自动安装当前仅支持 x64"
    );
    let source = format!(
        "https://github.com/artempyanykh/marksman/releases/download/{version}/marksman.exe"
    );
    let entry = stage.join("marksman.exe");
    http::download(&source, &entry, context)?;
    Ok((source, entry))
}
