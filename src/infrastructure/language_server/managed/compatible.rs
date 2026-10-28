use super::{InstallContext, http};
use anyhow::{Context, Result};
use node_semver::{Range, Version};
use std::{path::Path, process::Command};

pub(in crate::infrastructure::language_server) fn runtime(
    program: &Path,
    argument: &str,
) -> Result<String> {
    let mut command = Command::new(program);
    command.arg(argument);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let output = command.output()?;
    anyhow::ensure!(
        output.status.success(),
        "无法读取运行环境版本：{}",
        program.display()
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn supports_node(metadata: &serde_json::Value, node: &Version) -> Result<bool> {
    match metadata["engines"]["node"].as_str() {
        Some(required) => Ok(Range::parse(required)?.satisfies(node)),
        None => Ok(true),
    }
}

pub(super) fn npm(package: &str, node: &Version, context: &InstallContext) -> Result<String> {
    let metadata: serde_json::Value = serde_json::from_str(&http::text(&format!(
        "https://registry.npmjs.org/{package}/latest"
    ))?)?;
    let latest = metadata["version"].as_str().context("npm 缺少最新版本")?;
    if supports_node(&metadata, node)? {
        return Ok(latest.into());
    }
    context.progress(format!(
        "{package} 最新版不支持 Node {node}，正在选择兼容版本…"
    ))?;
    let releases: serde_json::Value = serde_json::from_str(&http::text_limit(
        &format!("https://registry.npmjs.org/{package}"),
        32 * 1024 * 1024,
    )?)?;
    let latest_version = Version::parse(latest)?;
    let mut versions = releases["versions"]
        .as_object()
        .context("npm 缺少版本列表")?
        .iter()
        .filter_map(|(text, metadata)| {
            if text.contains('-') || metadata["deprecated"].is_string() {
                return None;
            }
            let version = Version::parse(text).ok()?;
            (version <= latest_version).then_some((version, text, metadata))
        })
        .collect::<Vec<_>>();
    versions.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, text, metadata) in versions {
        context.check()?;
        if supports_node(metadata, node)? {
            context.selection(format!(
                "{package} 最新版 {latest} 不兼容 Node {node}，使用兼容版本 {text}"
            ))?;
            return Ok(text.clone());
        }
    }
    anyhow::bail!("没有兼容 Node {node} 的 {package} 版本，请升级 Node.js 后重试")
}

pub(in crate::infrastructure::language_server) fn numeric(version: &str) -> Result<Vec<u32>> {
    let mut parts = version
        .split('.')
        .map(str::parse)
        .collect::<std::result::Result<Vec<u32>, _>>()?;
    parts.resize(3, 0);
    Ok(parts)
}
