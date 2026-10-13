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
    let history: OpenHistory =
        serde_json::from_reader(file).context("打开历史格式损坏，原文件已保留")?;
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
    if history.session.as_ref().is_some_and(|saved| {
        same_path(&saved.project_root, &session.project_root) && saved.revision > session.revision
    }) {
        return Ok(history);
    }
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
            .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        left == right
    }
}
