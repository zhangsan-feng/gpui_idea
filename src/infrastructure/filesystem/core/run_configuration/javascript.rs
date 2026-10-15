use super::common::*;
use crate::domain::run::RunConfiguration;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{fs, path::Path};

pub(super) fn javascript(manifest: &Path) -> Result<Option<RunConfiguration>> {
    let package: Value =
        serde_json::from_slice(&fs::read(manifest)?).context("package.json 格式错误")?;
    let script = ["start", "dev"].into_iter().find(|name| {
        package["scripts"][name]
            .as_str()
            .is_some_and(|script| !script.trim().is_empty())
    });
    let directory = manifest.parent().unwrap();
    let manager = package["packageManager"]
        .as_str()
        .map(|manager| manager.split('@').next().unwrap_or("npm"))
        .filter(|manager| matches!(*manager, "pnpm" | "yarn" | "bun" | "npm"))
        .unwrap_or_else(|| {
            if directory.join("pnpm-lock.yaml").is_file() {
                "pnpm"
            } else if directory.join("yarn.lock").is_file() {
                "yarn"
            } else if directory.join("bun.lock").is_file() || directory.join("bun.lockb").is_file()
            {
                "bun"
            } else {
                "npm"
            }
        });
    let executable = if cfg!(windows) && manager != "bun" {
        format!("{manager}.cmd")
    } else {
        manager.to_string()
    };
    let build = package["scripts"]["build"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|_| in_directory(directory, &format!("{executable} run build")));
    let debug = package["scripts"]["debug"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|_| in_directory(directory, &format!("{executable} run debug")));
    if script.is_none() && build.is_none() && debug.is_none() {
        return Ok(None);
    }
    let mut result = configuration(
        format!("{manager} · {}", script.unwrap_or("项目")),
        script
            .map(|script| in_directory(directory, &format!("{executable} run {script}")))
            .unwrap_or_default(),
        build,
        debug,
    );
    result.profiles[0].name = script.unwrap_or("default").into();
    if manager == "npm" {
        result.profiles[0].argument_prefix = "--".into();
        result.profiles[0].debug_argument_prefix = "--".into();
    }
    for alternate in ["start", "dev"] {
        if Some(alternate) != script
            && package["scripts"][alternate]
                .as_str()
                .is_some_and(|s| !s.trim().is_empty())
        {
            let mut profile = result.profiles[0].clone();
            profile.name = alternate.into();
            profile.command = in_directory(directory, &format!("{executable} run {alternate}"));
            result.profiles.push(profile);
        }
    }
    Ok(Some(result))
}
pub(super) fn javascript_file(
    file: &Path,
    extension: &str,
    manifest: Option<&Path>,
) -> Result<RunConfiguration> {
    if matches!(extension, "js" | "mjs" | "cjs") {
        let directory = file.parent().unwrap();
        return Ok(configuration(
            "JavaScript · 文件",
            in_directory(directory, &format!("node {}", quote(file))),
            Some(in_directory(
                directory,
                &format!("node --check {}", quote(file)),
            )),
            Some(in_directory(
                directory,
                &format!("node inspect {}", quote(file)),
            )),
        ));
    }
    if let Some(manifest) = manifest {
        let runner = manifest.parent().unwrap().join("node_modules/.bin/tsx.cmd");
        if runner.is_file() && matches!(extension, "ts" | "mts" | "cts" | "tsx" | "jsx") {
            return Ok(configuration(
                "TypeScript / JSX · tsx",
                in_directory(
                    manifest.parent().unwrap(),
                    &format!("& {} {}", quote(runner), quote(file)),
                ),
                None,
                None,
            ));
        }
    }
    bail!(
        "此文件需要项目运行脚本或已安装的 tsx 运行器，请配置 package.json 的 start/dev 或手动输入命令"
    )
}
