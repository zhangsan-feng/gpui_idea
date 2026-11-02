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
    Ok(run_tools::interpreted_file(
        file.parent().context("文件没有父目录")?,
        root,
        file,
        "php",
    ))
}
