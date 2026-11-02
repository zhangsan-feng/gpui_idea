use crate::domain::run::RunConfigurationDataContextSnapshot;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(in crate::infrastructure::filesystem) fn runnable(root: &Path, file: &Path) -> bool {
    match crate::infrastructure::language_server::local_file_run_configuration(root, file) {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(error) => {
            tracing::debug!(file = %file.display(), %error, "File run target unavailable");
            false
        }
    }
}

pub(in crate::infrastructure::filesystem) fn resolve_file(
    root: PathBuf,
    relative: PathBuf,
) -> Result<RunConfigurationDataContextSnapshot> {
    let root = root.canonicalize()?;
    let file = super::super::path::resolve(&root, &relative, false)?;
    let mut result =
        crate::infrastructure::language_server::local_file_run_configuration(&root, &file)?
            .context("此文件没有可运行入口或运行器不可用")?;
    result.file = Some(relative);
    Ok(result)
}

pub(in crate::infrastructure) fn checked_file(root: &Path, relative: &Path) -> Result<PathBuf> {
    super::super::path::resolve(root, relative, false)
}
