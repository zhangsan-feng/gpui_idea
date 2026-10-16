use super::path::resolve;
use anyhow::{Result, ensure};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub(in crate::infrastructure::filesystem) fn validate_entry_batch(
    root: &Path,
    sources: &[PathBuf],
    target: Option<&Path>,
) -> Result<()> {
    let directory = target
        .map(|target| {
            let directory = if target.as_os_str().is_empty() {
                root.canonicalize()?
            } else {
                resolve(root, target, false)?
            };
            ensure!(directory.is_dir(), "移动目标必须是目录");
            Ok::<_, anyhow::Error>(directory)
        })
        .transpose()?;
    let mut destinations = HashSet::new();
    for source in sources {
        let from = resolve(root, source, false)?;
        if let Some(directory) = &directory {
            ensure!(
                !directory.starts_with(&from),
                "不能移入自身或子目录：{}",
                source.display()
            );
            let destination = directory.join(
                source
                    .file_name()
                    .ok_or_else(|| anyhow::anyhow!("不能移动项目根目录"))?,
            );
            ensure!(
                from != destination,
                "文件或目录已经位于目标目录：{}",
                source.display()
            );
            // Existing targets and same-name items from different parents must
            // be rejected before moving any member of the selection.
            match fs::symlink_metadata(&destination) {
                Ok(_) => anyhow::bail!("目标已存在，未覆盖：{}", destination.display()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
            let key = destination.to_string_lossy().into_owned();
            #[cfg(windows)]
            let key = key.to_lowercase();
            ensure!(
                destinations.insert(key),
                "选中项在目标目录重名：{}",
                source.display()
            );
        }
    }
    Ok(())
}
