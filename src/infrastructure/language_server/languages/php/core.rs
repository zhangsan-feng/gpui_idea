use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"completion": {"insertUseDeclaration": false, "triggerParameterHints": false}})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    let storage = storage();
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
    _root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    let command = managed::node_server(&NODE, context)?;
    for directory in ["storage", "global"] {
        std::fs::create_dir_all(storage().join(directory))?;
    }
    Ok(command)
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::node_server(&NODE, context).map(|_| ())
}

fn storage() -> std::path::PathBuf {
    crate::infrastructure::paths::get()
        .data
        .join("lsp/intelephense")
}
