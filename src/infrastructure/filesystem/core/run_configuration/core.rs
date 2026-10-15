use super::{common::*, interpreted, javascript, native, rust};
use crate::domain::run::RunConfiguration;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Language/toolchain detection and command generation are infrastructure IO.
/// The GUI receives capabilities, never constructs language-specific commands.
pub(in crate::infrastructure::filesystem) fn resolve(
    root: PathBuf,
    selected: Option<PathBuf>,
) -> Result<RunConfiguration> {
    let root = root.canonicalize().context("项目目录不可访问")?;
    let file = selected
        .as_ref()
        .map(|file| {
            let relative = if file.is_absolute() {
                file.canonicalize()?
                    .strip_prefix(&root)
                    .context("当前文件不在项目内")?
                    .to_owned()
            } else {
                file.clone()
            };
            super::super::path::resolve(&root, &relative, false)
        })
        .transpose()?;
    let directory = file.as_deref().and_then(Path::parent).unwrap_or(&root);
    let extension = file
        .as_deref()
        .and_then(Path::extension)
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    let found = |name| nearest(directory, &root, name);
    let mut result = match extension.as_str() {
        "rs" => {
            if let Some(manifest) = found("Cargo.toml") {
                rust::cargo(&root, &manifest, file.as_deref())?
            } else {
                native::rust_file(file.as_deref().unwrap())
            }
        }
        "go" => native::go(directory, found("go.mod").as_deref(), file.as_deref()),
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx" | "vue" | "svelte" => {
            let manifest = found("package.json");
            if let Some(config) = manifest
                .as_deref()
                .map(javascript::javascript)
                .transpose()?
                .flatten()
            {
                if config.command.is_empty() {
                    if let Ok(mut file_config) = javascript::javascript_file(
                        file.as_deref().unwrap(),
                        &extension,
                        manifest.as_deref(),
                    ) {
                        for profile in &mut file_config.profiles {
                            if let Some(project_profile) = config.profiles.first() {
                                if project_profile.build_command.is_some() {
                                    profile.build_command = project_profile.build_command.clone();
                                }
                                if project_profile.debug_command.is_some() {
                                    profile.debug_command = project_profile.debug_command.clone();
                                }
                            }
                        }
                        file_config
                    } else {
                        config
                    }
                } else {
                    config
                }
            } else {
                javascript::javascript_file(
                    file.as_deref().unwrap(),
                    &extension,
                    manifest.as_deref(),
                )?
            }
        }
        "c" | "cc" | "cpp" | "cxx" => {
            native::cpp(directory, &root, file.as_deref().unwrap(), &extension)?
        }
        "py" | "pyw" | "rb" | "php" | "ps1" | "sh" | "lua" | "pl" => {
            interpreted::resolve(directory, &root, file.as_deref().unwrap(), &extension)
        }
        _ => project(directory, &root)?,
    };
    result.file = selected;
    tracing::debug!(label = %result.label, profiles = result.profiles.len(), "toolchain capabilities resolved");
    Ok(result)
}

fn project(directory: &Path, root: &Path) -> Result<RunConfiguration> {
    let manifest = ["Cargo.toml", "package.json", "go.mod", "CMakeLists.txt"]
        .into_iter()
        .filter_map(|name| nearest(directory, root, name))
        .max_by_key(|path| path.components().count());
    if let Some(manifest) = manifest {
        let directory = manifest.parent().unwrap();
        return match manifest.file_name().and_then(|name| name.to_str()) {
            Some("Cargo.toml") => rust::cargo(root, &manifest, None),
            Some("package.json") => javascript::javascript(&manifest)?
                .context("package.json 中没有 start/dev 脚本，请选择代码文件或手动输入命令"),
            Some("go.mod") => Ok(native::go(directory, Some(&manifest), None)),
            Some("CMakeLists.txt") => native::cmake(directory),
            _ => unreachable!(),
        };
    }
    // Python projects need an explicit entry; arbitrary modules are never guessed.
    for name in ["__main__.py", "main.py", "app.py"] {
        if let Some(file) = nearest(directory, root, name) {
            return Ok(interpreted::resolve(
                file.parent().unwrap(),
                root,
                &file,
                "py",
            ));
        }
    }
    bail!("未识别到可运行的代码或项目，请选择代码文件或手动输入命令")
}
