use super::{InstallContext, Installed, compatible, core, update};
use anyhow::{Context, Result};
use std::{fs, path::Path, process::Command};

pub(in crate::infrastructure::language_server) fn node_server(
    package: &str,
    binary: &str,
    context: &InstallContext,
) -> Result<(tokio::process::Command, Option<String>)> {
    let group = match package {
        "pyright" => "python",
        "typescript-language-server" => "typescript",
        "vscode-langservers-extracted" => "web",
        "yaml-language-server" => "yaml",
        _ => anyhow::bail!("未知 Node 语言服务器：{package}"),
    };
    let node = super::super::launch::executable("node")
        .context("未找到 Node.js，请自行安装包含 npm 的 Node.js 并加入 PATH，然后重试语言服务")?;
    let node_version = node_semver::Version::parse(
        compatible::runtime(&node, "--version")?.trim_start_matches('v'),
    )?;
    let installed = update::ensure(
        &format!("lsp-{group}"),
        package,
        context,
        || {
            let version = compatible::npm(package, &node_version, context)?;
            if package == "typescript-language-server" {
                Ok(format!(
                    "{version}__{}",
                    typescript_version(&node, &version, context)?
                ))
            } else {
                Ok(version)
            }
        },
        |stage, release| {
            context.progress(format!("正在安装 {package}…"))?;
            let npm = npm_cli(&node)?;
            let mut command = Command::new(&node);
            command
                .arg(npm)
                .args([
                    "install",
                    "--save-exact",
                    "--ignore-scripts",
                    "--no-audit",
                    "--no-fund",
                    "--registry=https://registry.npmjs.org",
                ])
                .arg(format!("{package}@{}", release.split("__").next().unwrap()))
                .arg("--prefix")
                .arg(stage)
                .current_dir(stage)
                .env("npm_config_cache", core::root().join("npm-cache"))
                .env("npm_config_engine_strict", "true")
                .env("npm_config_update_notifier", "false");
            if package == "typescript-language-server" {
                command.arg(format!(
                    "typescript@{}",
                    release.split_once("__").context("TypeScript 版本无效")?.1
                ));
            }
            core::run(command, context, package).context(
            "请确认系统 Node.js/npm 满足服务器版本要求（详见安装日志），且可以访问 npm registry",
        )?;
            let (entry, version) = entry(stage, package, binary)?;
            Ok(Installed {
                entry: entry.strip_prefix(stage)?.to_owned(),
                version,
                source: format!("npm:{package}@{release}"),
                selection: None,
            })
        },
    )?;
    tracing::debug!(package, version = installed.version, path = %installed.path.display(), "Using managed LSP package");
    // Web servers share an installation but have distinct entry points.
    let directory = installed
        .path
        .ancestors()
        .find(|path| path.join("installed.json").is_file())
        .context("Node 服务器缓存目录无效")?;
    let (entry, version) = entry(directory, package, binary)?;
    let mut command = tokio::process::Command::new(&node);
    command.arg(entry).arg("--stdio");
    let paths = std::env::join_paths(
        std::iter::once(node.parent().unwrap().to_owned()).chain(
            std::env::var_os("PATH")
                .into_iter()
                .flat_map(|value| std::env::split_paths(&value).collect::<Vec<_>>()),
        ),
    )?;
    command.env("PATH", paths);
    Ok((command, Some(version)))
}

fn typescript_version(node: &Path, server: &str, context: &InstallContext) -> Result<String> {
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
    let mut command = Command::new(node);
    command.arg(npm_cli(node)?).args([
        "view",
        &format!("typescript@{major}"),
        "version",
        "--json",
        "--registry=https://registry.npmjs.org",
    ]);
    command.env("npm_config_update_notifier", "false");
    let output = core::capture(command, context, "查询兼容 TypeScript 版本")?;
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

fn npm_cli(node: &Path) -> Result<std::path::PathBuf> {
    let mut directories = vec![node.parent().context("Node 路径无效")?.to_owned()];
    if let Some(appdata) = std::env::var_os("APPDATA") {
        directories.push(std::path::PathBuf::from(appdata).join("npm"));
    }
    if let Some(paths) = std::env::var_os("PATH") {
        directories.extend(std::env::split_paths(&paths));
    }
    directories
        .into_iter()
        .map(|directory| directory.join("node_modules/npm/bin/npm-cli.js"))
        .find(|path| path.is_file())
        .context("未找到 npm，请安装包含 npm 的 Node.js 发行版后重试")
}

fn entry(directory: &Path, package: &str, binary: &str) -> Result<(std::path::PathBuf, String)> {
    let package = directory.join("node_modules").join(package);
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(package.join("package.json"))?)?;
    let entry = manifest["bin"]
        .as_str()
        .or_else(|| manifest["bin"][binary].as_str())
        .context("Node 语言服务器缺少入口")?;
    let entry = package.join(entry);
    anyhow::ensure!(
        entry.is_file(),
        "Node 语言服务器入口不存在：{}",
        entry.display()
    );
    Ok((
        entry,
        manifest["version"].as_str().unwrap_or("unknown").into(),
    ))
}
