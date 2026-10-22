use super::*;
use serde::{Deserialize, Serialize};
use std::io::{Read as _, Write as _};

const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct WorkspaceReference {
    pub(super) project_root: PathBuf,
    pub(super) file: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct HistoryIndex {
    pub(super) version: u32,
    pub(super) entries: Vec<OpenHistoryEntry>,
    pub(super) last_project: Option<PathBuf>,
    pub(super) workspaces: Vec<WorkspaceReference>,
}

impl Default for HistoryIndex {
    fn default() -> Self {
        Self {
            version: 2,
            entries: Vec::new(),
            last_project: None,
            workspaces: Vec::new(),
        }
    }
}

pub(super) fn read_json(path: &Path) -> Result<Option<serde_json::Value>> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("无法读取状态文件：{}", path.display()));
        }
    };
    if file.metadata()?.len() > MAX_FILE_BYTES {
        bail!("状态文件过大：{}", path.display());
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        bail!("状态文件在读取期间超过限制");
    }
    serde_json::from_slice(&bytes)
        .context("状态文件格式损坏，原文件已保留")
        .map(Some)
}

pub(super) fn load_index(path: &Path) -> Result<HistoryIndex> {
    let Some(value) = read_json(path)? else {
        return Ok(HistoryIndex::default());
    };
    let mut index: HistoryIndex = match value.get("version").and_then(|value| value.as_u64()) {
        Some(1) => super::migration::migrate(path, value)?,
        Some(2) => serde_json::from_value(value).context("项目历史索引格式无效")?,
        _ => bail!("不支持的打开历史版本"),
    };
    normalize_entries(&mut index.entries);
    // Older versions left this pointer behind when a recent project was removed.
    if index.last_project.as_ref().is_some_and(|root| {
        !index
            .entries
            .iter()
            .any(|entry| same_path(&entry.path, root))
    }) {
        tracing::info!(project = ?index.last_project, "ignoring startup project absent from recent history");
        index.last_project = None;
    }
    Ok(index)
}

pub(super) fn hydrate(path: &Path, index: HistoryIndex) -> Result<OpenHistory> {
    let mut sessions = Vec::new();
    for reference in index.workspaces.iter().take(LIMIT_SESSIONS) {
        match read_session(path, reference) {
            Ok(Some(session)) => sessions.push(session),
            Ok(None) => {
                tracing::warn!(project = %reference.project_root.display(), "workspace session file is missing")
            }
            Err(error) => {
                tracing::warn!(project = %reference.project_root.display(), %error, "workspace session could not be loaded; original file preserved")
            }
        }
    }
    Ok(OpenHistory {
        version: 2,
        entries: index.entries,
        last_project: index.last_project,
        sessions,
    })
}

fn session_path(index: &Path, reference: &WorkspaceReference) -> Result<PathBuf> {
    // References are filenames, never paths supplied to filesystem operations.
    if reference.file.len() != 21
        || !reference.file.ends_with(".json")
        || !reference.file.as_bytes()[..16]
            .iter()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("工作区状态文件名无效");
    }
    Ok(index
        .parent()
        .context("历史记录缺少目录")?
        .join("workspace-sessions")
        .join(&reference.file))
}

pub(super) fn session_name(project: &Path) -> String {
    // Stable FNV-1a key: independent of Rust's unspecified DefaultHasher.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in normalized_path(project).bytes() {
        hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}.json")
}

pub(super) fn read_session(
    path: &Path,
    reference: &WorkspaceReference,
) -> Result<Option<WorkspaceSession>> {
    let Some(value) = read_json(&session_path(path, reference)?)? else {
        return Ok(None);
    };
    let session: WorkspaceSession =
        serde_json::from_value(value).context("工作区恢复状态格式无效")?;
    if !same_path(&session.project_root, &reference.project_root) {
        bail!("工作区状态与项目路径不一致，原文件已保留");
    }
    Ok(Some(session))
}

pub(super) fn write_session(
    path: &Path,
    reference: &WorkspaceReference,
    session: &WorkspaceSession,
) -> Result<()> {
    // Refuse a hash collision or a mismatched existing reference rather than overwrite it.
    read_session(path, reference)?;
    write_json(&session_path(path, reference)?, session)
}

pub(super) fn remove_session(path: &Path, reference: &WorkspaceReference) {
    if read_session(path, reference).ok().flatten().is_none() {
        return;
    }
    if let Ok(target) = session_path(path, reference) {
        if let Err(error) = fs::remove_file(target) {
            tracing::warn!(%error, "expired workspace state cleanup failed");
        }
    }
}

pub(super) fn write_index(path: &Path, index: &HistoryIndex) -> Result<()> {
    write_json(path, index)
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::create_dir_all(path.parent().context("状态文件缺少存储目录")?)?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::File::create(&temporary).context("无法创建状态临时文件")?;
        serde_json::to_writer_pretty(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path).context("无法提交状态文件")
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
