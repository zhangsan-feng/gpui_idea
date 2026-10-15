use super::path::{is_link, resolve};
use anyhow::{Context, Result, bail};
use std::{
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    process::Command,
};

pub(in crate::infrastructure::filesystem) fn remove_entry(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    // resolve rejects the project root, root escapes and links before recursive deletion.
    let path = resolve(root, relative, false)?;
    let is_directory = fs::metadata(&path)?.is_dir();
    if is_directory {
        fs::remove_dir_all(path).context("删除目录失败")
    } else {
        fs::remove_file(path).context("删除文件失败")
    }
}

pub(in crate::infrastructure::filesystem) fn copy_destination(
    root: &Path,
    source: &Path,
    directory: &Path,
) -> Result<PathBuf> {
    let from = resolve(root, source, false)?;
    let target = if directory.as_os_str().is_empty() {
        root.canonicalize()?
    } else {
        resolve(root, directory, false)?
    };
    if !target.is_dir() {
        bail!("粘贴目标必须是目录");
    }
    if target.starts_with(&from) {
        bail!("不能将目录复制到自身或子目录");
    }
    let is_directory = from.is_dir();
    let original = source
        .file_name()
        .context("不能复制项目根目录")?
        .to_string_lossy();
    let stem = if is_directory {
        original.as_ref()
    } else {
        source
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or(&original)
    };
    let extension = if is_directory {
        None
    } else {
        source.extension().and_then(|name| name.to_str())
    };
    for index in 0..10000 {
        let name = if index == 0 {
            original.to_string()
        } else {
            let suffix = if index == 1 {
                " (副本)".to_owned()
            } else {
                format!(" (副本 {index})")
            };
            match extension {
                Some(extension) => format!("{stem}{suffix}.{extension}"),
                None => format!("{stem}{suffix}"),
            }
        };
        let destination = directory.join(name);
        let to = resolve(root, &destination, true)?;
        if fs::symlink_metadata(&to).is_ok() {
            continue;
        }
        return Ok(destination);
    }
    bail!("副本名称已用尽，请重命名后再粘贴")
}

pub(in crate::infrastructure::filesystem) fn copy_entry(
    root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<PathBuf> {
    let from = resolve(root, source, false)?;
    let to = resolve(root, destination, true)?;
    if to.starts_with(&from) {
        bail!("不能将目录复制到自身或子目录");
    }
    let is_directory = from.is_dir();
    // Reserve exactly the path registered by Application; never overwrite a race winner.
    if is_directory {
        fs::create_dir(&to).context("无法创建副本，目标可能已存在")?;
    } else {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&to)
            .context("无法创建副本，目标可能已存在")?;
    }
    let result = if is_directory {
        copy_directory(&from, &to)
    } else {
        copy_contents(&from, &to)
    };
    if let Err(error) = result {
        if is_directory {
            let _ = fs::remove_dir_all(&to);
        } else {
            let _ = fs::remove_file(&to);
        }
        return Err(error).context("复制失败，已清理未完成的副本");
    }
    Ok(destination.to_owned())
}

fn copy_contents(from: &Path, to: &Path) -> Result<()> {
    let mut source = fs::File::open(from)?;
    let mut destination = OpenOptions::new().write(true).open(to)?;
    std::io::copy(&mut source, &mut destination)?;
    destination.sync_all()?;
    Ok(())
}

fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    let mut pending = vec![(from.to_owned(), to.to_owned())];
    while let Some((source, destination)) = pending.pop() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if is_link(&metadata) {
                bail!("目录包含符号链接或目录连接，不能复制");
            }
            let target = destination.join(entry.file_name());
            if metadata.is_dir() {
                fs::create_dir(&target)?;
                pending.push((path, target));
            } else if metadata.is_file() {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&target)?;
                copy_contents(&path, &target)?;
            } else {
                bail!("目录包含不支持复制的文件类型");
            }
        }
    }
    Ok(())
}

pub(in crate::infrastructure::filesystem) fn reveal_entry(
    root: &Path,
    relative: &Path,
) -> Result<()> {
    if !relative.as_os_str().is_empty() {
        resolve(root, relative, false)?;
    } else {
        root.canonicalize()?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let path = root.join(relative);
        let mut explorer = Command::new("explorer.exe");
        explorer.creation_flags(0x08000000);
        if relative.as_os_str().is_empty() {
            explorer.arg(path);
        } else {
            let mut argument = std::ffi::OsString::from("/select,");
            argument.push(path);
            explorer.arg(argument);
        }
        explorer.spawn().context("无法打开文件所在目录")?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        bail!("当前仅支持 Windows 文件资源管理器");
    }
}
