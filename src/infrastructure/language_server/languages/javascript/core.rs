use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Context;
use anyhow::Result;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"suggest": {"autoImports": false, "completeFunctionCalls": false}})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    json!({"preferences": {"includeCompletionsForModuleExports": false, "includeCompletionsWithInsertText": true, "includeCompletionsWithSnippetText": true}})
}

pub(in crate::infrastructure::language_server) const NODE: managed::node::NodeServer =
    managed::node::NodeServer {
        key: "lsp-typescript",
        package: "typescript-language-server",
        binary: "typescript-language-server",
        companion: Some(("typescript", typescript_version)),
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

pub(in crate::infrastructure::language_server) fn typescript_version(
    node: &Path,
    server: &str,
    context: &InstallContext,
) -> Result<String> {
    // TLS 6 documents TypeScript 6; TLS 5 uses the JavaScript TypeScript 5 API.
    // TypeScript 7's native compiler is not a drop-in tsserver.js replacement.
    let major = if server
        .split('.')
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
        >= 6
    {
        6
    } else {
        5
    };
    let mut command = ProcessCommand::new(node);
    command.arg(managed::node::npm_cli(node)?).args([
        "view",
        &format!("typescript@{major}"),
        "version",
        "--json",
        "--registry=https://registry.npmjs.org",
    ]);
    command.env("npm_config_update_notifier", "false");
    let output = managed::core::capture(command, context, "查询兼容 TypeScript 版本")?;
    let value: serde_json::Value = serde_json::from_slice(&fs::read(output)?)?;
    let versions: Vec<&str> = match &value {
        serde_json::Value::String(value) => vec![value],
        serde_json::Value::Array(values) => {
            values.iter().filter_map(|value| value.as_str()).collect()
        }
        _ => anyhow::bail!("npm TypeScript 版本列表无效"),
    };
    versions
        .into_iter()
        .filter_map(|text| {
            node_semver::Version::parse(text)
                .ok()
                .map(|version| (version, text))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, text)| text.to_owned())
        .context("没有兼容的 TypeScript 版本")
}
