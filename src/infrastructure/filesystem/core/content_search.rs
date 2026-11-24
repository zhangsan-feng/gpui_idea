use super::super::directory_filter::DirectoryFilter;
use crate::domain::project::{ContentMatch, ContentSearchDataContextSnapshot};
use anyhow::{Context, Result};
use std::{fs, io::Read, path::PathBuf, time::Instant};
use tokio::sync::watch;

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_VISITED: usize = 100_000;
const MAX_MATCHES: usize = 100;

pub(in crate::infrastructure::filesystem) fn search_content(
    root: PathBuf,
    query: String,
    filter: &DirectoryFilter,
    cancel: &watch::Receiver<()>,
) -> Result<Option<ContentSearchDataContextSnapshot>> {
    if cancel.has_changed().is_err() {
        return Ok(None);
    }
    let mut snapshot = ContentSearchDataContextSnapshot::default();
    if query.is_empty() {
        return Ok(Some(snapshot));
    }
    let started = Instant::now();
    let root = root.canonicalize().context("项目目录不可访问")?;
    let app_paths = crate::infrastructure::paths::get();
    let app_directories = [app_paths.data.clone(), app_paths.tools.join("managed")]
        .map(|path| path.canonicalize().ok());
    let mut pending = vec![root.clone()];
    let mut visited = 0;
    let mut files_read = 0;
    let mut bytes = Vec::new();
    'scan: while let Some(directory) = pending.pop() {
        if cancel.has_changed().is_err() {
            return Ok(None);
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(path = %directory.display(), %error, "content search directory skipped");
                continue;
            }
        };
        for entry in entries {
            if cancel.has_changed().is_err() {
                return Ok(None);
            }
            if visited == MAX_VISITED {
                snapshot.truncated = true;
                break 'scan;
            }
            visited += 1;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if super::path::is_link(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                let name = entry.file_name();
                if filter.matches_name(&name)
                    || matches!(
                        name.to_str(),
                        Some(
                            ".git"
                                | "target"
                                | "node_modules"
                                | "dist"
                                | "build"
                                | "artifacts"
                                | "logs"
                                | ".codex"
                                | ".agents"
                                | ".venv"
                                | "venv"
                                | "__pycache__"
                                | ".gpui-run"
                        )
                    )
                {
                    continue;
                }
                let path = entry.path();
                if !app_directories
                    .iter()
                    .flatten()
                    .any(|ignored| ignored == &path)
                {
                    pending.push(path);
                }
                continue;
            }
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
                continue;
            }
            let path = entry.path();
            bytes.clear();
            files_read += 1;
            let result = fs::File::open(&path)
                .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes));
            if result.is_err() || bytes.len() as u64 > MAX_FILE_BYTES {
                continue;
            }
            let Ok(text) = std::str::from_utf8(&bytes) else {
                continue;
            };
            let text = text.strip_prefix('\u{feff}').unwrap_or(text);
            if !text.contains(&query) || bytes.contains(&0) {
                continue;
            }
            let relative = path.strip_prefix(&root)?;
            for (index, line) in text.lines().enumerate() {
                if let Some(offset) = line.find(&query) {
                    let column = line[..offset].chars().count();
                    snapshot.matches.push(ContentMatch {
                        path: relative.to_owned(),
                        line: index as u32 + 1,
                        column: column as u32 + 1,
                        preview: line
                            .chars()
                            .skip(column.saturating_sub(40))
                            .take(180)
                            .collect(),
                    });
                    if snapshot.matches.len() >= MAX_MATCHES {
                        snapshot.truncated = true;
                        break 'scan;
                    }
                }
            }
        }
    }
    snapshot
        .matches
        .sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    tracing::debug!(
        visited,
        files_read,
        elapsed_ms = started.elapsed().as_millis(),
        "content search scan completed"
    );
    Ok(Some(snapshot))
}
