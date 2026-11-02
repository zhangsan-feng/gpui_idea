use super::super::{FileRunSupport, ServerRunSupport};
use crate::{
    domain::run::{RunConfigurationDataContextSnapshot, RunProfile},
    infrastructure::filesystem::run_tools,
};
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::path::Path;

pub(super) const SUPPORT: FileRunSupport = FileRunSupport {
    accepts_file,
    resolve,
    server: Some(ServerRunSupport {
        method: "experimental/runnables",
        capability: "/capabilities/experimental/runnables",
        parameters: |uri| serde_json::json!({"textDocument": {"uri": uri}, "position": null}),
        parse: parse_server,
    }),
};

fn accepts_file(file: &Path) -> bool {
    file.file_name().is_some_and(|name| name == "main.rs")
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfigurationDataContextSnapshot> {
    // Avoid invoking Cargo once for every Rust module when a directory is expanded.
    let source = std::fs::read_to_string(file)?;
    static MAIN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let main = MAIN.get_or_init(|| regex::Regex::new(r"\bfn\s+main\s*\(").unwrap());
    ensure!(main.is_match(&source), "Rust 文件没有 main 入口");
    let manifest = run_tools::nearest(file.parent().context("文件没有父目录")?, root, "Cargo.toml")
        .context("Rust 文件所在项目缺少 Cargo.toml")?;
    run_tools::cargo_entry(root, &manifest, file)
}

fn strings(value: &Value) -> Result<Vec<String>> {
    value
        .as_array()
        .context("语言服务运行参数格式无效")?
        .iter()
        .map(|value| {
            let value = value.as_str().context("语言服务运行参数不是字符串")?;
            ensure!(
                !value.contains(['\0', '\r', '\n']),
                "语言服务运行参数包含非法字符"
            );
            Ok(value.to_owned())
        })
        .collect()
}

fn parse_server(
    root: &Path,
    response: &Value,
) -> Result<Option<RunConfigurationDataContextSnapshot>> {
    let items = response.as_array().context("无法解析语言服务运行目标")?;
    let runnable = items
        .iter()
        .find(|item| item["kind"] == "cargo" && item["args"]["cargoArgs"][0] == "run");
    let Some(runnable) = runnable else {
        return Ok(None);
    };
    let args = &runnable["args"];
    let directory = args["cwd"]
        .as_str()
        .or_else(|| args["workspaceRoot"].as_str())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.to_owned())
        .canonicalize()?;
    ensure!(
        directory.starts_with(root.canonicalize()?),
        "语言服务运行目录不在当前工作区内"
    );
    let mut cargo = strings(&args["cargoArgs"])?;
    // Older rust-analyzer versions return these separately.
    if args["cargoExtraArgs"].is_array() {
        cargo.extend(strings(&args["cargoExtraArgs"])?);
    }
    let executable = if args["executableArgs"].is_array() {
        strings(&args["executableArgs"])?
    } else {
        vec![]
    };
    let program = args["overrideCargo"].as_str().unwrap_or("cargo");
    ensure!(
        !program.contains(['\0', '\r', '\n']),
        "语言服务运行程序包含非法字符"
    );
    let mut environment = String::new();
    if let Some(variables) = args["environment"].as_object() {
        for (key, value) in variables {
            let value = value.as_str().context("语言服务环境变量格式无效")?;
            ensure!(
                !key.is_empty()
                    && key
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    && !value.contains(['\0', '\r', '\n', '"']),
                "语言服务环境变量包含非法字符"
            );
            if cfg!(windows) {
                environment.push_str(&format!("set \"{key}={}\" && ", value.replace('%', "%%")));
            } else {
                environment.push_str(&format!("{key}={} ", run_tools::quote(value)));
            }
        }
    }
    let command = |release: bool, build: bool| {
        let mut cargo = cargo.clone();
        if build {
            cargo[0] = "build".into();
        }
        if release && !cargo.iter().any(|arg| arg == "--release") {
            cargo.push("--release".into());
        }
        let mut command = format!(
            "{} {}",
            run_tools::quote(program),
            cargo
                .iter()
                .map(run_tools::quote)
                .collect::<Vec<_>>()
                .join(" ")
        );
        if !build {
            command.push_str(" --");
            if !executable.is_empty() {
                command.push(' ');
                command.push_str(
                    &executable
                        .iter()
                        .map(run_tools::quote)
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        run_tools::in_directory(&directory, &format!("{environment}{command}"))
    };
    let profiles = [false, true]
        .into_iter()
        .map(|release| RunProfile {
            name: if release { "release" } else { "dev" }.into(),
            command: command(release, false),
            build_command: Some(command(release, true)),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    Ok(Some(RunConfigurationDataContextSnapshot {
        label: format!("Rust · {}", runnable["label"].as_str().unwrap_or("运行")),
        command: profiles[0].command.clone(),
        profiles,
        ..Default::default()
    }))
}
