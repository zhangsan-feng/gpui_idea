use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"validate": true})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    json!({"provideFormatter": false})
}

pub(in crate::infrastructure::language_server) const NODE: managed::node::NodeServer =
    managed::node::NodeServer {
        key: "lsp-web",
        package: "vscode-langservers-extracted",
        binary: "vscode-css-language-server",
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
