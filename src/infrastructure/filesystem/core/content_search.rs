use crate::domain::project::{ContentMatch, ContentSearchSnapshot};
use anyhow::{Context, Result};
use std::{fs, io::Read, path::PathBuf};

pub(in crate::infrastructure::filesystem) fn search_content(
    root: PathBuf,
    query: String,
) -> Result<ContentSearchSnapshot> {
    let mut snapshot = ContentSearchSnapshot::default();
    if query.is_empty() {
        return Ok(snapshot);
    }
    let root = root.canonicalize().context("项目目录不可访问")?;
    let mut pending = vec![root.clone()];
    let mut visited = 0;
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(path = %directory.display(), %error, "content search directory skipped");
                continue;
            }
        };
        for entry in entries {
            visited += 1;
            if visited > 100_000 {
                snapshot.truncated = true;
                return Ok(snapshot);
            }
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if super::path::is_link(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                if !matches!(
                    entry.file_name().to_str(),
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
                ) {
                    pending.push(path);
                }
                continue;
            }
            if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
                continue;
            }
            let mut bytes = Vec::new();
            let result = fs::File::open(&path)
                .and_then(|file| file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes));
            if result.is_err() || bytes.len() > 2 * 1024 * 1024 || bytes.contains(&0) {
                continue;
            }
            let Ok(text) = std::str::from_utf8(&bytes) else {
                continue;
            };
            let text = text.strip_prefix('\u{feff}').unwrap_or(text);
            for (index, line) in text.lines().enumerate() {
                if let Some(offset) = line.find(&query) {
                    let prefix = &line[..offset];
                    snapshot.matches.push(ContentMatch {
                        path: path.strip_prefix(&root)?.to_owned(),
                        line: index as u32 + 1,
                        column: prefix.chars().count() as u32 + 1,
                        preview: line
                            .chars()
                            .skip(prefix.chars().count().saturating_sub(40))
                            .take(180)
                            .collect(),
                    });
                    if snapshot.matches.len() >= 100 {
                        snapshot.truncated = true;
                        return Ok(snapshot);
                    }
                }
            }
        }
    }
    snapshot
        .matches
        .sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
    Ok(snapshot)
}
