use crate::infrastructure::language_server::managed::{self, InstallContext};
use anyhow::Context;
use anyhow::Result;
use managed::compatible::numeric;
use managed::{Installed, compatible, core, http, update};
use serde_json::{Value, json};
use std::path::Path;
use std::{fs, path::PathBuf};
use tokio::process::Command;

pub(in crate::infrastructure::language_server) fn configuration(_section: &str) -> Value {
    json!({"completeUnimported": false, "usePlaceholders": false})
}
pub(in crate::infrastructure::language_server) fn initialization_options() -> Value {
    configuration("")
}

fn command(program: &std::path::Path) -> std::process::Command {
    let mut command = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command
}
pub(in crate::infrastructure::language_server) fn server(
    context: &InstallContext,
) -> Result<PathBuf> {
    let go = crate::infrastructure::language_server::launch::executable("go")
        .context("未找到 Go 环境，请安装 Go SDK 并将 go 加入 PATH 后重试；程序会自动安装 gopls")?;
    let binary = update::ensure(
        "gopls",
        "gopls",
        context,
        || match &context.requested_release {
            Some(version) => Ok(version.clone()),
            None => compatible_release(&compatible::runtime(&go, "version")?, context),
        },
        |stage, version| {
            context.progress("正在使用本机 Go 安装 gopls…")?;
            fs::create_dir_all(stage.join("tmp"))?;
            let mut install = command(&go);
            install
                .args(["install", &format!("golang.org/x/tools/gopls@{version}")])
                .env("GOBIN", stage.join("bin"))
                .env("GOPATH", core::root().join("go-path"))
                .env("GOMODCACHE", core::root().join("go-mod-cache"))
                .env("GOCACHE", core::root().join("go-build-cache"))
                .env("GOTMPDIR", stage.join("tmp"))
                // Never let go install implicitly fetch another Go SDK.
                .env("GOTOOLCHAIN", "local")
                .env("GOWORK", "off")
                .env("GO111MODULE", "on")
                .env("GOFLAGS", "")
                .current_dir(stage);
            core::run(install, context, "Go 安装 gopls")
                .context("请确认本机 Go SDK 满足最新版 gopls 要求；版本过旧时请升级 Go 后重试")?;
            Ok(Installed {
                entry: if cfg!(windows) {
                    "bin/gopls.exe"
                } else {
                    "bin/gopls"
                }
                .into(),
                version: version.into(),
                source: format!("go:golang.org/x/tools/gopls@{version}"),
                selection: None,
            })
        },
    )?;
    Ok(binary.path)
}
pub(in crate::infrastructure::language_server) fn compatible_release(
    current: &str,
    context: &InstallContext,
) -> Result<String> {
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
pub(in crate::infrastructure::language_server) fn prepare(
    _root: &Path,
    context: &InstallContext,
) -> Result<(Command, Option<String>)> {
    Ok((Command::new(server(context)?), None))
}
pub(in crate::infrastructure::language_server) fn install(context: &InstallContext) -> Result<()> {
    server(context).map(|_| ())
}
