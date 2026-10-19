use super::{Binary, core, management, update};
use anyhow::{Context, Result, ensure};
use std::{cmp::Ordering, fs, path::PathBuf, time::SystemTime};

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
    let mut removed = 0;
    for (key, _, _) in management::SERVERS {
        // Serialize cleanup with installation and version selection.
        let slot = update::slot(key);
        let mut checked = slot.lock().unwrap_or_else(|error| error.into_inner());
        let mut versions = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let directory = entry.file_name().to_string_lossy().into_owned();
            if directory != *key && !directory.starts_with(&format!("{key}--")) {
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
        *checked = Some(latest.binary);
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
        "Historical LSP versions cleaned; only latest installed versions retained"
    );
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
