use super::{common::nearest, interpreted, javascript, native, rust};
use crate::domain::run::RunConfiguration;
use anyhow::{Context, Result, ensure};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::OnceLock,
};

fn go_main(file: &Path) -> bool {
    if file
        .file_name()
        .is_some_and(|name| name.to_string_lossy().ends_with("_test.go"))
    {
        return false;
    }
    let mut bytes = Vec::new();
    if File::open(file)
        .and_then(|file| file.take(64 * 1024).read_to_end(&mut bytes))
        .is_err()
    {
        return false;
    }
    static PACKAGE: OnceLock<regex::Regex> = OnceLock::new();
    let pattern = PACKAGE.get_or_init(|| {
        regex::Regex::new(
        r"(?s)\A(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)*package(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)+main\b"
    ).expect("valid package declaration pattern")
    });
    pattern.is_match(String::from_utf8_lossy(&bytes).trim_start_matches('\u{feff}'))
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfiguration> {
    let directory = file.parent().context("文件没有父目录")?;
    let extension = file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "py" | "pyw" => Ok(interpreted::resolve(directory, root, file, &extension)),
        "js" | "mjs" | "cjs" => javascript::javascript_file(file, &extension, None),
        "go" => {
            ensure!(go_main(file), "仅 package main 的非测试 Go 文件支持运行");
            Ok(native::go(
                directory,
                nearest(directory, root, "go.mod").as_deref(),
                Some(file),
            ))
        }
        "rs" => {
            ensure!(
                file.file_name().is_some_and(|name| name == "main.rs"),
                "Rust 仅支持在 main.rs 上运行 Cargo"
            );
            let manifest = nearest(directory, root, "Cargo.toml")
                .context("main.rs 所在项目缺少 Cargo.toml")?;
            rust::cargo_entry(root, &manifest, file)
        }
        _ => anyhow::bail!("文件树运行暂支持 Python、Go、JavaScript 和 Rust"),
    }
}

pub(in crate::infrastructure::filesystem) fn runnable(root: &Path, file: &Path) -> bool {
    if !crate::domain::run::supports_file(file) {
        return false;
    }
    let extension = file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let runnable = match extension.as_str() {
        "py" | "pyw" | "js" | "mjs" | "cjs" => true,
        "go" => go_main(file),
        "rs" if file.file_name().is_some_and(|name| name == "main.rs") => {
            resolve(root, file).is_ok()
        }
        _ => false,
    };
    tracing::debug!(file = %file.display(), runnable, "file run capability detected");
    runnable
}

pub(in crate::infrastructure::filesystem) fn resolve_file(
    root: PathBuf,
    relative: PathBuf,
) -> Result<RunConfiguration> {
    let root = root.canonicalize()?;
    let file = super::super::path::resolve(&root, &relative, false)?;
    let mut result = resolve(&root, &file)?;
    result.file = Some(relative);
    Ok(result)
}
