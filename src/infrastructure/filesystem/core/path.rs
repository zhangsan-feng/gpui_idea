use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(in crate::infrastructure::filesystem) fn resolve(
    root: &Path,
    relative: &Path,
    allow_new: bool,
) -> Result<PathBuf> {
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("操作必须指定项目内的文件或目录，不能修改项目根目录");
    }
    let root = root.canonicalize().context("项目根目录不可访问")?;
    let mut path = root.clone();
    let parts: Vec<_> = relative.components().collect();
    for (index, part) in parts.iter().enumerate() {
        path.push(part.as_os_str());
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if is_link(&metadata) {
                    bail!("不能操作符号链接或目录连接");
                }
                if !path.canonicalize()?.starts_with(&root) {
                    bail!("路径超出项目范围");
                }
            }
            Err(error)
                if allow_new
                    && index + 1 == parts.len()
                    && error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("文件或父目录不可访问"),
        }
    }
    Ok(path)
}

pub(crate) fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}
