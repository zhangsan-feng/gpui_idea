use super::common::*;
use crate::domain::run::{RunConfiguration, RunProfile};
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{path::Path, process::Command};

pub(super) fn cargo(
    root: &Path,
    manifest: &Path,
    selected: Option<&Path>,
) -> Result<RunConfiguration> {
    let mut process = Command::new("cargo");
    crate::infrastructure::process_environment::sanitize(&mut process);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        process.creation_flags(0x08000000);
    }
    let output = process
        .args([
            "metadata",
            "--no-deps",
            "--offline",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest)
        .current_dir(manifest.parent().unwrap())
        .output()
        .context("无法调用 Cargo，请检查 Rust 工具链")?;
    if !output.status.success() {
        bail!(
            "Cargo 项目识别失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let packages = metadata["packages"]
        .as_array()
        .context("Cargo metadata 缺少 packages")?;
    let package = packages
        .iter()
        .find(|package| {
            package["manifest_path"].as_str().is_some_and(|path| {
                Path::new(path).canonicalize().ok() == manifest.canonicalize().ok()
            })
        })
        .or_else(|| {
            let defaults = metadata["workspace_default_members"].as_array()?;
            let mut matches = packages
                .iter()
                .filter(|package| defaults.contains(&package["id"]));
            let first = matches.next()?;
            matches.next().is_none().then_some(first)
        });
    let Some(package) = package else {
        return Ok(build_only(root, manifest, "Rust · workspace", None));
    };
    let targets = package["targets"]
        .as_array()
        .context("Cargo 包缺少 targets")?;
    let kind = |target: &Value, name: &str| {
        target["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == name))
    };
    let selected_target = selected.and_then(|file| {
        targets
            .iter()
            .filter_map(|target| {
                if !(kind(target, "bin") || kind(target, "example")) {
                    return None;
                }
                let path = Path::new(target["src_path"].as_str()?)
                    .canonicalize()
                    .ok()?;
                if file == path {
                    Some((usize::MAX, target))
                } else if path.file_name().is_some_and(|name| name == "main.rs")
                    && path.parent().is_some_and(|parent| file.starts_with(parent))
                {
                    Some((path.components().count(), target))
                } else {
                    None
                }
            })
            .max_by_key(|(priority, _)| *priority)
            .map(|(_, target)| target)
    });
    let bins: Vec<_> = targets
        .iter()
        .filter(|target| kind(target, "bin"))
        .collect();
    let target = selected_target.or_else(|| {
        let default = package["default_run"].as_str().unwrap_or_default();
        bins.iter()
            .copied()
            .find(|target| target["name"].as_str() == Some(default))
            .or_else(|| (bins.len() == 1).then(|| bins[0]))
    });
    let package_name = package["name"].as_str().context("Cargo 包缺少名称")?;
    let Some(target) = target else {
        return Ok(build_only(
            root,
            manifest,
            &format!("Rust · {package_name}"),
            Some(package_name),
        ));
    };
    let name = target["name"].as_str().context("Cargo 目标缺少名称")?;
    let option = if kind(target, "example") {
        "--example"
    } else {
        "--bin"
    };
    let command = format!(
        "cargo run --color=always --manifest-path {} --package {} {option} {}",
        quote(manifest.file_name().unwrap()),
        quote(package_name),
        quote(name)
    );
    let directory = manifest.parent().unwrap();
    let wrap = |command: &str| {
        if directory == root {
            command.to_owned()
        } else {
            in_directory(directory, command)
        }
    };
    let build = command.replacen("cargo run", "cargo build", 1);
    let mut debug_argument_template = None;
    let debug = if cfg!(windows) {
        None
    } else {
        let debugger = executable("rust-gdb").or_else(|| executable("rust-lldb"));
        debugger.map(|debugger| {
            let target_directory =
                Path::new(metadata["target_directory"].as_str().unwrap_or("target"));
            let mut binary = target_directory.join("debug");
            if kind(target, "example") {
                binary = binary.join("examples");
            }
            let binary = binary.join(name);
            let prefix = if debugger.to_string_lossy().contains("lldb") {
                "--"
            } else {
                "--args"
            };
            debug_argument_template = Some(format!(
                "{} && {} {prefix} {} __GPUI_PROGRAM_ARGUMENTS__",
                wrap(&build),
                quote(&debugger),
                quote(&binary)
            ));
            format!(
                "{} && {} {prefix} {}",
                wrap(&build),
                quote(debugger),
                quote(binary)
            )
        })
    };
    Ok(RunConfiguration {
        command: wrap(&command),
        label: format!("Rust · {name}"),
        profiles: vec![
            RunProfile {
                name: "dev".into(),
                command: wrap(&command),
                build_command: Some(wrap(&build)),
                debug_command: debug,
                argument_prefix: "--".into(),
                argument_template: None,
                debug_argument_prefix: String::new(),
                debug_argument_template,
            },
            RunProfile {
                name: "release".into(),
                command: wrap(&format!("{command} --release")),
                build_command: Some(wrap(&format!("{build} --release"))),
                debug_command: None,
                argument_prefix: "--".into(),
                argument_template: None,
                debug_argument_prefix: String::new(),
                debug_argument_template: None,
            },
        ],
        ..Default::default()
    })
}

fn build_only(
    root: &Path,
    manifest: &Path,
    label: &str,
    package: Option<&str>,
) -> RunConfiguration {
    let target = package
        .map(|name| format!("--package {}", quote(name)))
        .unwrap_or_else(|| "--workspace".into());
    let build = format!(
        "cargo build --color=always --manifest-path {} {target}",
        quote(manifest)
    );
    let wrap = |command: &str| {
        if manifest.parent() == Some(root) {
            command.to_owned()
        } else {
            in_directory(manifest.parent().unwrap(), command)
        }
    };
    RunConfiguration {
        label: label.into(),
        profiles: ["dev", "release"]
            .into_iter()
            .map(|name| RunProfile {
                name: name.into(),
                command: String::new(),
                build_command: Some(wrap(&if name == "release" {
                    format!("{build} --release")
                } else {
                    build.clone()
                })),
                debug_command: None,
                argument_prefix: String::new(),
                argument_template: None,
                debug_argument_prefix: String::new(),
                debug_argument_template: None,
            })
            .collect(),
        ..Default::default()
    }
}
