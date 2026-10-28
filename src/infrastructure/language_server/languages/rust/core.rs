use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use managed::{
    download::{architecture, extract_zip},
    http,
};
use serde_json::{Value, json};
use std::path::Path;
use std::{fs, path::PathBuf};
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"cargo": {"sysroot": "discover"}, "checkOnSave": false, "completion": {"autoimport": {"enable": false}}})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    configuration("")
}

pub(in crate::infrastructure::language_server) fn prepare(
    _root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    let command = Command::new(managed::download_server("rust-analyzer", context)?);
    Ok((command, None))
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::download_server("rust-analyzer", context).map(|_| ())
}

pub(in crate::infrastructure::language_server) fn latest_release(
    _context: &InstallContext,
) -> Result<String> {
    http::latest("rust-lang/rust-analyzer")
}
pub(in crate::infrastructure::language_server) fn download_release(
    stage: &Path,
    version: &str,
    context: &InstallContext,
) -> Result<(String, PathBuf)> {
    let arch = architecture()?;
    let source = format!(
        "https://github.com/rust-lang/rust-analyzer/releases/download/{version}/rust-analyzer-{arch}-pc-windows-msvc.zip"
    );
    let archive = stage.join("release.zip");
    http::download(&source, &archive, context)?;
    extract_zip(&archive, stage, context)?;
    fs::remove_file(archive)?;
    Ok((source, stage.join("rust-analyzer.exe")))
}
