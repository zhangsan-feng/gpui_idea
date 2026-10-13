use super::path::resolve;
use anyhow::{Context, Result};
use std::{fs, path::Path};

pub(in crate::infrastructure::filesystem) fn create_directory(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    fs::create_dir(resolve(root, relative, true)?).context("新建目录失败")
}
pub(in crate::infrastructure::filesystem) fn remove_directory(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    // Only empty directories are removed; callers cannot silently delete a tree.
    fs::remove_dir(resolve(root, relative, false)?).context("删除目录失败，目录必须为空")
}
