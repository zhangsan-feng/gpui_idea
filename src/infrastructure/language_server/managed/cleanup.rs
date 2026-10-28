use super::{Binary, core, management, update};
use anyhow::{Context, Result, ensure};
use std::{
    cmp::Ordering,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

struct Version {
    directory: String,
    path: PathBuf,
    binary: Binary,
    modified: Option<SystemTime>,
}

pub(in crate::infrastructure::language_server) fn clean_history() -> Result<()> {
    let root = core::root();
    if !root.try_exists()? {
        return Ok(());
    }
    let resolved_root = root.canonicalize().context("无法定位 LSP 安装目录")?;
    // npm installations share a cache across server families. Hold every
    // installation slot so no background installer can use a cache we remove.
    let slots = super::super::languages::managed()
        .map(|language| update::slot(language.key))
        .collect::<Vec<_>>();
    let mut guards = slots
        .iter()
        .map(|slot| slot.lock().unwrap_or_else(|error| error.into_inner()))
        .collect::<Vec<_>>();
    let removed_caches = clean_caches(&root, &resolved_root)?;
    let mut removed = 0;
    for (language, checked) in super::super::languages::managed().zip(guards.iter_mut()) {
        let key = language.key;
        let mut versions = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let directory = entry.file_name().to_string_lossy().into_owned();
            if directory != key && !directory.starts_with(&format!("{key}--")) {
                continue;
            }
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                continue;
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    continue;
                }
            }
            // Only complete managed installations participate in cleanup.
            let Some(binary) = core::cached(&path) else {
                continue;
            };
            let path = path.canonicalize()?;
            ensure!(
                path.parent() == Some(resolved_root.as_path()),
                "LSP 历史版本目录超出安装目录"
            );
            versions.push(Version {
                directory,
                path,
                binary,
                modified: metadata.modified().ok(),
            });
        }
        versions.sort_by(|a, b| {
            compare_release(&a.binary.version, &b.binary.version)
                .then_with(|| compare_release(&a.directory, &b.directory))
                .then_with(|| a.modified.cmp(&b.modified))
                .then_with(|| a.directory.cmp(&b.directory))
        });
        let Some(latest) = versions.pop() else {
            continue;
        };
        // Preserve fixed mode, redirecting an old selection to the retained version.
        if management::pin(key)?.is_some_and(|pin| pin != latest.directory) {
            management::save_pin(key, &Some(latest.directory.clone()))?;
        }
        **checked = Some(latest.binary);
        tracing::info!(
            key,
            directory = latest.directory,
            "Retaining latest installed LSP version"
        );
        for old in versions {
            tracing::info!(
                key,
                directory = old.directory,
                "Removing historical LSP version"
            );
            fs::remove_dir_all(&old.path).with_context(|| {
                format!("无法清理 LSP 历史版本 {}；文件被占用时请关闭使用旧版本的项目后重试，部分版本可能已清理，请刷新列表", old.directory)
            })?;
            removed += 1;
        }
    }
    tracing::info!(
        removed,
        removed_caches,
        "Historical LSP versions cleaned; only latest installed versions retained"
    );
    Ok(())
}

fn clean_caches(root: &Path, resolved_root: &Path) -> Result<usize> {
    let mut removed = 0;
    for name in ["go-build-cache", "go-mod-cache", "go-path", "npm-cache"] {
        let path = root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error).with_context(|| format!("无法读取 LSP 缓存 {name}")),
        };
        ensure!(
            is_directory(&metadata),
            "LSP 缓存 {name} 不是普通目录，未清理"
        );
        let resolved = path.canonicalize()?;
        ensure!(
            resolved.parent() == Some(resolved_root),
            "LSP 缓存目录超出安装目录"
        );
        tracing::info!(directory = name, "Removing LSP installation cache");
        // Go's module cache contains read-only source files/directories.
        make_removable(&resolved)
            .and_then(|()| fs::remove_dir_all(&resolved))
            .with_context(|| format!("无法清理 LSP 缓存 {name}；文件被占用时请稍后重试"))?;
        removed += 1;
    }
    Ok(removed)
}

fn is_directory(metadata: &fs::Metadata) -> bool {
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    true
}

fn make_removable(path: &Path) -> std::io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Ok(());
        }
        if metadata.permissions().readonly() {
            let mut permissions = metadata.permissions();
            permissions.set_readonly(false);
            fs::set_permissions(path, permissions)?;
        }
    }
    #[cfg(unix)]
    if metadata.is_dir() {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o700);
        fs::set_permissions(path, permissions)?;
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            make_removable(&entry?.path())?;
        }
    }
    Ok(())
}

fn compare_release(a: &str, b: &str) -> Ordering {
    if let (Ok(a), Ok(b)) = (
        node_semver::Version::parse(a.trim_start_matches('v')),
        node_semver::Version::parse(b.trim_start_matches('v')),
    ) {
        return a.cmp(&b);
    }
    // Date releases and compound TypeScript versions also need numeric ordering.
    let numbers = |value: &str| {
        value
            .split(|c: char| !c.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.parse::<u64>().ok())
            .collect::<Vec<_>>()
    };
    numbers(a).cmp(&numbers(b))
}
