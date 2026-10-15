use crate::domain::history::{OpenHistory, OpenHistoryEntry, OpenKind, WorkspaceSession};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const LIMIT_PER_KIND: usize = 100;
const MAX_HISTORY_BYTES: u64 = 4 * 1024 * 1024;

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

pub(super) fn forget_project(path: &Path, project: &Path) -> Result<OpenHistory> {
    let mut history = read(path)?;
    history
        .entries
        .retain(|entry| entry.kind != OpenKind::Project || !same_path(&entry.path, project));
    write(path, &history)?;
    Ok(history)
}

pub(super) fn read(path: &Path) -> Result<OpenHistory> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(OpenHistory::default());
        }
        Err(error) => return Err(error).context("无法读取打开历史"),
    };
    if file.metadata()?.len() > MAX_HISTORY_BYTES {
        bail!("打开历史文件过大");
    }
    let mut value: serde_json::Value =
        serde_json::from_reader(file).context("打开历史格式损坏，原文件已保留")?;
    if let Some(session) = value
        .get_mut("session")
        .and_then(serde_json::Value::as_object_mut)
    {
        if !session.contains_key("run_automatic") {
            let command = session
                .get("run_command")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .trim();
            // Migrate the old default to detection, while preserving user overrides.
            let automatic = command.is_empty() || command == "cargo run";
            session.insert("run_automatic".into(), serde_json::Value::Bool(automatic));
        }
    }
    let history: OpenHistory =
        serde_json::from_value(value).context("打开历史格式损坏，原文件已保留")?;
    if history.version != 1 {
        bail!("不支持的打开历史版本：{}", history.version);
    }
    Ok(history)
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
    let target = validate_target(kind, target)?;
    let mut history = read(path)?;
    history
        .entries
        .retain(|entry| entry.kind != kind || !same_path(&entry.path, &target));
    let opened_at_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
    history.entries.insert(
        0,
        OpenHistoryEntry {
            kind,
            path: target,
            opened_at_ms,
        },
    );
    let mut counts = [0usize; 3];
    history.entries.retain(|entry| {
        let index = match entry.kind {
            OpenKind::Project => 0,
            OpenKind::Directory => 1,
            OpenKind::File => 2,
        };
        counts[index] += 1;
        counts[index] <= LIMIT_PER_KIND
    });
    write(path, &history)?;
    Ok(history)
}

pub(super) fn save_session(path: &Path, session: WorkspaceSession) -> Result<OpenHistory> {
    let mut history = read(path)?;
    // A slow background write cannot replace a newer snapshot from this workspace.
    if history
        .sessions
        .iter()
        .find(|saved| same_path(&saved.project_root, &session.project_root))
        .or_else(|| {
            history
                .session
                .as_ref()
                .filter(|saved| same_path(&saved.project_root, &session.project_root))
        })
        .is_some_and(|saved| {
            same_path(&saved.project_root, &session.project_root)
                && saved.revision > session.revision
        })
    {
        return Ok(history);
    }
    history
        .sessions
        .retain(|saved| !same_path(&saved.project_root, &session.project_root));
    history.sessions.insert(0, session.clone());
    history.sessions.truncate(30);
    history.session = Some(session);
    write(path, &history)?;
    Ok(history)
}

fn write(path: &Path, history: &OpenHistory) -> Result<()> {
    let directory = path.parent().context("打开历史缺少存储目录")?;
    fs::create_dir_all(directory).context("无法创建打开历史目录")?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::File::create(&temporary).context("无法创建历史临时文件")?;
        serde_json::to_writer_pretty(&mut file, history)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path).context("无法提交打开历史")
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(windows)]
    {
        left.as_os_str()
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .eq_ignore_ascii_case(
                right
                    .as_os_str()
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\"),
            )
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}
