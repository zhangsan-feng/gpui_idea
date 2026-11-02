use super::super::FileRunSupport;
use crate::{
    domain::run::RunConfigurationDataContextSnapshot, infrastructure::filesystem::run_tools,
};
use anyhow::{Context, Result};
use std::path::Path;

pub(super) const SUPPORT: FileRunSupport = FileRunSupport {
    accepts_file,
    resolve,
    server: None,
};

fn accepts_file(file: &Path) -> bool {
    !file
        .file_name()
        .is_some_and(|name| name.to_string_lossy().ends_with("_test.go"))
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfigurationDataContextSnapshot> {
    anyhow::ensure!(
        !file
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with("_test.go")),
        "Go 测试文件不是程序入口"
    );
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(file)?
        .take(64 * 1024)
        .read_to_end(&mut bytes)?;
    static PACKAGE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = PACKAGE.get_or_init(|| regex::Regex::new(r"(?s)\A(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)*package(?:\s|//[^\n]*(?:\n|$)|/\*.*?\*/)+main\b").unwrap());
    anyhow::ensure!(
        pattern.is_match(String::from_utf8_lossy(&bytes).trim_start_matches('\u{feff}')),
        "Go 文件不属于 package main"
    );
    let directory = file.parent().context("文件没有父目录")?;
    Ok(run_tools::go_file(
        directory,
        run_tools::nearest(directory, root, "go.mod").as_deref(),
        Some(file),
    ))
}
