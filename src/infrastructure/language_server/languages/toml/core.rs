use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use managed::{
    download::{architecture, copy_limited},
    http,
};
use serde_json::{Value, json};
use std::path::Path;
use std::{
    fs::{self, File},
    path::PathBuf,
};
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
    let mut command = Command::new(managed::download_server("taplo", context)?);
    command.args(["lsp", "stdio"]);
    Ok((command, None))
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::download_server("taplo", context).map(|_| ())
}

pub(in crate::infrastructure::language_server) fn latest_release(
    _context: &InstallContext,
) -> Result<String> {
    http::latest("tamasfe/taplo")
}
pub(in crate::infrastructure::language_server) fn download_release(
    stage: &Path,
    version: &str,
    context: &InstallContext,
) -> Result<(String, PathBuf)> {
    let arch = architecture()?;
    let source = format!(
        "https://github.com/tamasfe/taplo/releases/download/{version}/taplo-windows-{arch}.gz"
    );
    let archive = stage.join("release.gz");
    http::download(&source, &archive, context)?;
    let entry = stage.join("taplo.exe");
    copy_limited(
        flate2::read::GzDecoder::new(File::open(&archive)?),
        &entry,
        context,
    )?;
    fs::remove_file(archive)?;
    Ok((source, entry))
}
