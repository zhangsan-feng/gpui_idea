use super::{InstallContext, Installed, compatible, core, update};
use anyhow::{Context, Result};
use std::{fs, path::Path, process::Command};

type CompanionVersion = fn(&Path, &str, &InstallContext) -> Result<String>;

#[derive(Clone, Copy)]
pub(in crate::infrastructure::language_server) struct NodeServer {
    pub key: &'static str,
    pub package: &'static str,
    pub binary: &'static str,
    pub companion: Option<(&'static str, CompanionVersion)>,
}

pub(in crate::infrastructure::language_server) fn node_server(
    server: &NodeServer,
    context: &InstallContext,
) -> Result<(tokio::process::Command, Option<String>)> {
    let NodeServer {
        package, binary, ..
    } = *server;
    let node = super::super::launch::executable("node")
        .context("未找到 Node.js，请自行安装包含 npm 的 Node.js 并加入 PATH，然后重试语言服务")?;
    let node_version = node_semver::Version::parse(
        compatible::runtime(&node, "--version")?.trim_start_matches('v'),
    )?;
    let installed = update::ensure(
        server.key,
        package,
        context,
        || {
            let version = match &context.requested_release {
                Some(version) => version.clone(),
                None => compatible::npm(package, &node_version, context)?,
            };
            if let Some((_, select_version)) = server.companion {
                Ok(format!(
                    "{version}__{}",
                    select_version(&node, &version, context)?
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
            if let Some((companion, _)) = server.companion {
                command.arg(format!(
                    "{companion}@{}",
                    release
                        .split_once("__")
                        .context("语言服务器依赖版本无效")?
                        .1
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

pub(in crate::infrastructure::language_server) fn npm_cli(
    node: &Path,
) -> Result<std::path::PathBuf> {
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
