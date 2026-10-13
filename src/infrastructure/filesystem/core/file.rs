use super::path::resolve;
use crate::infrastructure::filesystem::FileMetadata;
use anyhow::{Context, Result, bail};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};

pub(in crate::infrastructure::filesystem) fn metadata(
    root: &Path,
    relative: &Path,
) -> Result<FileMetadata> {
    let path = resolve(root, relative, false)?;
    let metadata = fs::metadata(path)?;
    Ok(FileMetadata {
        relative_path: relative.to_owned(),
        is_directory: metadata.is_dir(),
        bytes: metadata.len(),
        readonly: metadata.permissions().readonly(),
        modified: metadata.modified().ok(),
    })
}
pub(in crate::infrastructure::filesystem) fn read_file(
    root: &Path,
    relative: &Path,
    max_bytes: usize,
) -> Result<Vec<u8>> {
    let path = resolve(root, relative, false)?;
    let file = fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        bail!("目标不是普通文件");
    }
    if file.metadata()?.len() > max_bytes as u64 {
        bail!("文件超过读取上限");
    }
    let mut bytes = Vec::new();
    file.take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        bail!("文件在读取期间超过上限");
    }
    Ok(bytes)
}
pub(in crate::infrastructure::filesystem) fn create_file(
    root: &Path,
    relative: &Path,
    contents: &[u8],
) -> Result<()> {
    let path = resolve(root, relative, true)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .context("新建失败，目标可能已存在")?;
    let result = file.write_all(contents).and_then(|_| file.sync_all());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    Ok(result?)
}
pub(in crate::infrastructure::filesystem) fn write_file(
    root: &Path,
    relative: &Path,
    contents: &[u8],
    expected: &[u8],
) -> Result<()> {
    super::write_changed_block(root, relative, contents, expected)
}

pub(in crate::infrastructure::filesystem) fn rename(
    root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<()> {
    let from = resolve(root, source, false)?;
    let to = resolve(root, destination, true)?;
    if fs::symlink_metadata(&to).is_ok() {
        bail!("重命名目标已存在，未覆盖");
    }
    if to.starts_with(&from) {
        bail!("目录不能移动到自身内部");
    }
    fs::rename(from, to).context("重命名或移动失败")
}
pub(in crate::infrastructure::filesystem) fn remove_file(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    let path = resolve(root, relative, false)?;
    if !fs::metadata(&path)?.is_file() {
        bail!("目标不是普通文件");
    }
    fs::remove_file(path).context("删除文件失败")
}
pub(in crate::infrastructure::filesystem) fn set_readonly(
    root: &Path,
    relative: &Path,
    readonly: bool,
) -> Result<()> {
    let path = resolve(root, relative, false)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_readonly(readonly);
    fs::set_permissions(path, permissions).context("修改只读属性失败")
}
