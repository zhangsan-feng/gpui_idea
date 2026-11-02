use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"completion": {"insertUseDeclaration": false, "triggerParameterHints": false}})
}
pub(in crate::infrastructure::language_server) fn initialization_options(
    root: &std::path::Path,
) -> Value {
    let storage = storage(root);
    json!({"storagePath": storage.join("storage"), "globalStoragePath": storage.join("global")})
}

pub(in crate::infrastructure::language_server) const NODE: managed::node::NodeServer =
    managed::node::NodeServer {
        key: "lsp-php",
        package: "intelephense",
        binary: "intelephense",
        companion: None,
    };
pub(in crate::infrastructure::language_server) fn prepare(
    root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    let command = managed::node_server(&NODE, context)?;
    for directory in ["storage", "global"] {
        std::fs::create_dir_all(storage(root).join(directory))?;
    }
    Ok(command)
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::node_server(&NODE, context).map(|_| ())
}

fn storage(root: &Path) -> std::path::PathBuf {
    crate::infrastructure::paths::project_data_directory(
        &crate::infrastructure::paths::get().data,
        root,
    )
    .join("lsp/intelephense")
}

pub(super) fn detect_workspace(root: &std::path::Path) -> bool {
    super::LANGUAGE
        .markers
        .iter()
        .any(|marker| root.join(marker).is_file())
}
