use super::{InstallContext, http};
use anyhow::{Context, Result};
use node_semver::{Range, Version};
use std::{path::Path, process::Command};

pub(super) fn runtime(program: &Path, argument: &str) -> Result<String> {
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

pub(super) fn go(current: &str, context: &InstallContext) -> Result<String> {
    let pattern = regex::Regex::new(r"go(\d+\.\d+(?:\.\d+)?)")?;
    let current_text = pattern.captures(current).context("无法识别 Go SDK 版本")?[1].to_owned();
    let current = numeric(&current_text)?;
    let base = "https://proxy.golang.org/golang.org/x/tools/gopls";
    let latest: serde_json::Value = serde_json::from_str(&http::text(&format!("{base}/@latest"))?)?;
    let latest = latest["Version"].as_str().context("缺少 gopls 版本")?;
    let mut versions = vec![latest.to_owned()];
    let mut fetched_history = false;
    let mut index = 0;
    let mut latest_required = String::new();
    loop {
        context.check()?;
        let version = &versions[index];
        let module = http::text(&format!("{base}/@v/{version}.mod"))?;
        let required = module
            .lines()
            .find_map(|line| line.strip_prefix("go "))
            .context("gopls 缺少 Go 版本要求")?;
        if index == 0 {
            latest_required = required.trim().to_owned();
        }
        if numeric(required.trim())? <= current {
            if version != latest {
                context.selection(format!(
                    "gopls 最新版 {latest} 要求 Go {latest_required}+，当前 Go {current_text}，使用兼容版本 {version}"
                ))?;
            }
            return Ok(version.clone());
        }
        if !fetched_history {
            let listing = http::text(&format!("{base}/@v/list"))?;
            let mut history = listing
                .lines()
                .filter(|line| !line.contains('-'))
                .filter_map(|line| {
                    Some((numeric(line.trim_start_matches('v')).ok()?, line.to_owned()))
                })
                .collect::<Vec<_>>();
            history.sort_by(|a, b| b.0.cmp(&a.0));
            versions.extend(
                history
                    .into_iter()
                    .map(|(_, version)| version)
                    .filter(|version| version != latest),
            );
            fetched_history = true;
        }
        index += 1;
        anyhow::ensure!(
            index < versions.len(),
            "没有兼容本机 Go SDK 的 gopls，请升级 Go 后重试"
        );
    }
}

pub(super) fn numeric(version: &str) -> Result<Vec<u32>> {
    let mut parts = version
        .split('.')
        .map(str::parse)
        .collect::<std::result::Result<Vec<u32>, _>>()?;
    parts.resize(3, 0);
    Ok(parts)
}

pub(super) fn java(versions: &[String], current: u32, context: &InstallContext) -> Result<String> {
    let latest = versions.last().context("Eclipse 目录没有 JDT LS 版本")?;
    if current >= 25 {
        return Ok(latest.clone());
    }
    // Eclipse release READMEs can lag behind platform bundle JVM requirements.
    // Select a known compatible release line directly from the installed JDK.
    let ceiling = if current >= 21 {
        "1.60.0"
    } else if current >= 17 {
        "1.43.0"
    } else {
        "1.26.0"
    };
    let ceiling = numeric(ceiling)?;
    let selected = versions
        .iter()
        .rev()
        .find(|version| numeric(version).is_ok_and(|version| version <= ceiling))
        .context("Eclipse 目录没有兼容本机 JDK 的 JDT LS 版本")?;
    context.selection(format!(
        "当前 JDK {current}，使用匹配版本 JDT LS {selected}（最新发行版 {latest}）"
    ))?;
    Ok(selected.clone())
}
