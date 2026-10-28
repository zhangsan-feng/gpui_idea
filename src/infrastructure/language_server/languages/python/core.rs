use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(section: &str) -> Value {
    let python = json!({"analysis": {"autoImportCompletions": false, "diagnosticMode": "openFilesOnly", "useLibraryCodeForTypes": true}});
    if section == "python.analysis" {
        python["analysis"].clone()
    } else {
        python
    }
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    json!({})
}

pub(in crate::infrastructure::language_server) const NODE: managed::node::NodeServer =
    managed::node::NodeServer {
        key: "lsp-python",
        package: "pyright",
        binary: "pyright-langserver",
        companion: None,
    };
pub(in crate::infrastructure::language_server) fn prepare(
    _root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    managed::node_server(&NODE, context)
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::node_server(&NODE, context).map(|_| ())
}
