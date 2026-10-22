mod migration;
mod storage;

use crate::domain::history::{OpenHistory, OpenHistoryEntry, OpenKind, WorkspaceSession};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use storage::{HistoryIndex, WorkspaceReference};

const LIMIT_PROJECTS: usize = 100;
const LIMIT_SESSIONS: usize = 30;

pub(super) fn lock_history(path: &Path) -> Result<fs::File> {
    fs::create_dir_all(path.parent().context("历史记录缺少存储目录")?)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))
        .context("无法创建项目历史锁")?;
    file.lock().context("无法锁定项目历史")?;
    Ok(file)
}

pub(super) fn read(path: &Path) -> Result<OpenHistory> {
    storage::hydrate(path, storage::load_index(path)?)
}

pub(super) fn prune_expanded_directories(mut session: WorkspaceSession) -> WorkspaceSession {
    session.expanded_directories.retain(|relative| {
        // Only inspect project-relative paths, even if a caller bypasses session validation.
        if relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return false;
        }
        let path = session.project_root.join(relative);
        let stale = match fs::metadata(&path) {
            Ok(metadata) => !metadata.is_dir(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "restored directory validation failed; keeping directory for retry");
                false
            }
        };
        if stale {
            tracing::info!(path = %path.display(), "stale expanded directory removed from restored session");
        }
        !stale
    });
    session
}

pub(super) fn forget_project(path: &Path, project: &Path) -> Result<OpenHistory> {
    let mut index = storage::load_index(path)?;
    index
        .entries
        .retain(|entry| !same_path(&entry.path, project));
    // Keep tabs/layout for an explicit reopen, but stop automatic restoration.
    if index
        .last_project
        .as_ref()
        .is_some_and(|root| same_path(root, project))
    {
        index.last_project = None;
    }
    storage::write_index(path, &index)?;
    tracing::info!(project = %project.display(), "recent project removed; automatic restoration disabled for this project");
    storage::hydrate(path, index)
}

pub(super) fn validate_target(kind: OpenKind, path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        bail!("打开历史必须使用绝对路径");
    }
    let metadata =
        fs::metadata(path).with_context(|| format!("路径不可访问：{}", path.display()))?;
    let valid = match kind {
        OpenKind::File => metadata.is_file(),
        _ => metadata.is_dir(),
    };
    if !valid {
        bail!("历史路径类型不匹配：{}", path.display());
    }
    path.canonicalize().context("无法解析历史路径")
}

pub(super) fn record(path: &Path, kind: OpenKind, target: &Path) -> Result<OpenHistory> {
    if kind != OpenKind::Project {
        return read(path);
    }
    let target = validate_target(kind, target)?;
    let mut index = storage::load_index(path)?;
    index
        .entries
        .retain(|entry| !same_path(&entry.path, &target));
    index.entries.insert(
        0,
        OpenHistoryEntry {
            kind,
            path: target,
            opened_at_ms: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
        },
    );
    normalize_entries(&mut index.entries);
    storage::write_index(path, &index)?;
    storage::hydrate(path, index)
}

pub(super) fn save_session(path: &Path, session: WorkspaceSession) -> Result<OpenHistory> {
    let mut index = storage::load_index(path)?;
    let previous = index.clone();
    let existing = index
        .workspaces
        .iter()
        .find(|item| same_path(&item.project_root, &session.project_root));
    let file = if let Some(reference) = existing {
        if let Some(saved) = storage::read_session(path, reference)? {
            if saved.revision > session.revision {
                return storage::hydrate(path, index);
            }
        }
        reference.file.clone()
    } else {
        storage::session_name(&session.project_root)
    };
    let reference = WorkspaceReference {
        project_root: session.project_root.clone(),
        file,
    };
    storage::write_session(path, &reference, &session)?;
    index
        .workspaces
        .retain(|item| !same_path(&item.project_root, &session.project_root));
    index.workspaces.insert(0, reference);
    let expired = if index.workspaces.len() > LIMIT_SESSIONS {
        index.workspaces.split_off(LIMIT_SESSIONS)
    } else {
        Vec::new()
    };
    // An already-open window can keep saving after its recent entry was removed.
    // Only explicitly opening the project again may make it restorable at startup.
    if index
        .entries
        .iter()
        .any(|entry| same_path(&entry.path, &session.project_root))
    {
        index.last_project = Some(session.project_root);
    }
    // Frequent cursor/layout saves rewrite only this project's session file.
    if index != previous {
        storage::write_index(path, &index)?;
    }
    for item in expired {
        storage::remove_session(path, &item);
    }
    storage::hydrate(path, index)
}

fn normalize_entries(entries: &mut Vec<OpenHistoryEntry>) {
    entries.retain(|entry| entry.kind == OpenKind::Project);
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.opened_at_ms));
    let mut roots = Vec::<PathBuf>::new();
    entries.retain(|entry| {
        if roots.iter().any(|root| same_path(root, &entry.path)) {
            false
        } else {
            roots.push(entry.path.clone());
            true
        }
    });
    entries.truncate(LIMIT_PROJECTS);
}

fn normalized_path(path: &Path) -> String {
    crate::domain::history::project_identity(path)
}

fn same_path(left: &Path, right: &Path) -> bool {
    normalized_path(left) == normalized_path(right)
}
