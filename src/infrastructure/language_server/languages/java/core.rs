use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"autobuild": {"enabled": false}, "completion": {"guessMethodArguments": false}})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    json!({"settings": {"java": configuration("")}})
}

pub(in crate::infrastructure::language_server) fn prepare(
    root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    Ok((super::launch::command(root, context)?, None))
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    managed::download_server("jdtls", context).map(|_| ())
}
