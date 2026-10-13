use crate::domain::project::ProjectEntry;
use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Component, Path},
};

pub(super) fn read(root: &Path, relative: &Path) -> Result<Vec<ProjectEntry>> {
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("无效的项目目录路径");
    }
    let path = if relative.as_os_str().is_empty() {
        root.canonicalize().context("无法读取项目根目录")?
    } else {
        super::super::core::path::resolve(root, relative, false)?
    };
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).context("无法读取目录")? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        let name = entry.file_name();
        entries.push(ProjectEntry {
            relative_path: relative.join(&name),
            name: name.to_string_lossy().into_owned(),
            is_directory: metadata.is_dir(),
            is_link: super::super::core::path::is_link(&metadata),
        });
    }
    entries.sort_by_cached_key(|entry| {
        (
            !entry.is_directory,
            entry.name.to_lowercase(),
            entry.relative_path.clone(),
        )
    });
    Ok(entries)
}
