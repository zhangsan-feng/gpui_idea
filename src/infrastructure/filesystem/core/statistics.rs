use crate::domain::statistics::{ExtensionStatistics, StatisticsSnapshot};
use anyhow::{Context, Result};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub(in crate::infrastructure::filesystem) fn statistics(
    root: PathBuf,
) -> Result<StatisticsSnapshot> {
    let root = root.canonicalize().context("无法读取项目目录")?;
    let mut stack = vec![root];
    let mut groups = BTreeMap::<String, ExtensionStatistics>::new();
    let mut snapshot = StatisticsSnapshot::default();
    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(path=%directory.display(),%error,"statistics directory skipped");
                snapshot.skipped_files += 1;
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    snapshot.skipped_files += 1;
                    continue;
                }
            };
            let path = entry.path();
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(_) => {
                    snapshot.skipped_files += 1;
                    continue;
                }
            };
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                let name = entry.file_name();
                if !matches!(
                    name.to_str(),
                    Some(
                        ".git"
                            | ".idea"
                            | "target"
                            | "node_modules"
                            | "dist"
                            | "build"
                            | "artifacts"
                            | ".codex"
                            | ".agents"
                            | ".venv"
                            | "__pycache__"
                            | ".gpui-run"
                    )
                ) {
                    stack.push(path);
                }
            } else if kind.is_file() {
                match measure_file(&path) {
                    Ok((bytes, lines, code)) => {
                        let extension = path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("[no extension]")
                            .to_ascii_lowercase();
                        let group = groups.entry(extension.clone()).or_insert_with(|| {
                            ExtensionStatistics {
                                extension,
                                ..Default::default()
                            }
                        });
                        group.add(bytes, lines, code);
                        snapshot.total.add(bytes, lines, code);
                    }
                    Err(error) => {
                        snapshot.skipped_files += 1;
                        tracing::warn!(path=%path.display(),%error,"statistics file skipped");
                    }
                }
            }
        }
    }
    snapshot.extensions = groups.into_values().collect();
    snapshot.total.extension = "Total".into();
    snapshot.scanned_at = chrono::Local::now().format("%H:%M:%S").to_string();
    tracing::debug!(
        files = snapshot.total.count,
        lines = snapshot.total.lines,
        groups = snapshot.extensions.len(),
        skipped = snapshot.skipped_files,
        "project statistics scan completed"
    );
    Ok(snapshot)
}

fn measure_file(path: &Path) -> Result<(u64, u64, u64)> {
    let file = File::open(path)?;
    let bytes = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut lines = 0;
    let mut non_empty_lines = 0;
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        // Binary files still contribute their type, count and size.
        // Detect them by content rather than a fixed extension list.
        let text = match std::str::from_utf8(&line) {
            Ok(text)
                if !text
                    .chars()
                    .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\r' | '\n' | '\u{c}')) =>
            {
                text
            }
            _ => return Ok((bytes, 0, 0)),
        };
        lines += 1;
        if !text.trim().trim_start_matches('\u{feff}').trim().is_empty() {
            non_empty_lines += 1;
        }
    }
    Ok((bytes, lines, non_empty_lines))
}
