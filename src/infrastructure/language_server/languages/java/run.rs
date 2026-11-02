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
    super::LANGUAGE.supports_file(file)
}

fn resolve(root: &Path, file: &Path) -> Result<RunConfigurationDataContextSnapshot> {
    let source = std::fs::read_to_string(file)?;
    static MAIN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let main = MAIN.get_or_init(|| {
        regex::Regex::new(r"\b(?:public\s+static|static\s+public)\s+void\s+main\s*\(").unwrap()
    });
    anyhow::ensure!(main.is_match(&source), "Java 文件没有 main 入口");
    Ok(run_tools::interpreted_file(
        file.parent().context("文件没有父目录")?,
        root,
        file,
        "java",
    ))
}
