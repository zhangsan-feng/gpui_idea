use super::{DirectoryScan, ScanResult, ScanTask, measure};
use crate::domain::statistics::{ExtensionStatistics, StatisticsSnapshot};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};
use tokio::task::JoinSet;

pub(super) async fn scan(root: PathBuf) -> Result<StatisticsSnapshot> {
    let started = Instant::now();
    let root = tokio::task::spawn_blocking(move || root.canonicalize())
        .await
        .context("统计目录解析任务失败")?
        .context("无法读取项目目录")?;
    let concurrency = std::thread::available_parallelism()
        .map(|count| count.get().saturating_mul(2))
        .unwrap_or(4)
        .clamp(4, 32);
    tracing::debug!(path = %root.display(), concurrency, "project statistics scan started");
    let mut pending = vec![ScanTask::Directory(root)];
    let mut running = JoinSet::new();
    let mut groups = BTreeMap::<String, ExtensionStatistics>::new();
    let mut snapshot = StatisticsSnapshot::default();
    let mut directories = 0_u64;

    loop {
        // Bound both active tasks and open files; collect each result immediately.
        while running.len() < concurrency {
            let Some(task) = pending.pop() else {
                break;
            };
            running.spawn(async move {
                // Native filesystem calls and text decoding must not block Tokio workers.
                tokio::task::spawn_blocking(move || execute(task)).await
            });
        }
        let Some(result) = running.join_next().await else {
            break;
        };
        match result
            .context("统计扫描协程失败")?
            .context("统计文件读取任务失败")?
        {
            ScanResult::Directory(result) => {
                directories += 1;
                pending.extend(result.tasks);
                snapshot.skipped_files += result.skipped;
            }
            ScanResult::File {
                extension,
                bytes,
                lines,
                code,
            } => {
                let group =
                    groups
                        .entry(extension.clone())
                        .or_insert_with(|| ExtensionStatistics {
                            extension,
                            ..Default::default()
                        });
                group.add(bytes, lines, code);
                snapshot.total.add(bytes, lines, code);
            }
            ScanResult::Skipped => snapshot.skipped_files += 1,
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
        directories,
        concurrency,
        elapsed_ms = started.elapsed().as_millis(),
        "project statistics scan completed"
    );
    Ok(snapshot)
}

fn execute(task: ScanTask) -> ScanResult {
    match task {
        ScanTask::Directory(path) => ScanResult::Directory(read_directory(path)),
        ScanTask::File(path) => match measure::file(&path) {
            Ok((bytes, lines, code)) => ScanResult::File {
                extension: path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("[no extension]")
                    .to_ascii_lowercase(),
                bytes,
                lines,
                code,
            },
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "statistics file skipped");
                ScanResult::Skipped
            }
        },
    }
}

fn read_directory(path: PathBuf) -> DirectoryScan {
    let mut result = DirectoryScan::default();
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(path = %path.display(), %error, "statistics directory skipped");
            result.skipped += 1;
            return result;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "statistics directory entry skipped");
                result.skipped += 1;
                continue;
            }
        };
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(error) => {
                tracing::warn!(path = %entry.path().display(), %error, "statistics file type skipped");
                result.skipped += 1;
                continue;
            }
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if !matches!(
                entry.file_name().to_str(),
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
                result.tasks.push(ScanTask::Directory(entry.path()));
            }
        } else if kind.is_file() {
            result.tasks.push(ScanTask::File(entry.path()));
        }
    }
    result
}
