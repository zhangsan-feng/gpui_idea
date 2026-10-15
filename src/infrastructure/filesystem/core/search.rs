use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

pub(in crate::infrastructure::filesystem) fn search_files(
    root: PathBuf,
    query: String,
) -> Result<Vec<PathBuf>> {
    let root = root.canonicalize().context("项目目录不可访问")?;
    let query = query.to_lowercase().replace('\\', "/");
    let mut stack = vec![root.clone()];
    let mut files = Vec::new();
    let mut visited = 0;
    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(path = %directory.display(), %error, "file search directory skipped");
                continue;
            }
        };
        for entry in entries {
            let entry = entry?;
            visited += 1;
            // Keep a very large/vendor-heavy workspace from monopolizing the IO pool.
            if visited > 100_000 {
                break;
            }
            let metadata = fs::symlink_metadata(entry.path())?;
            if super::path::is_link(&metadata) {
                continue;
            }
            let path = entry.path();
            if metadata.is_dir() {
                if !matches!(
                    entry.file_name().to_str(),
                    Some(
                        ".git"
                            | "target"
                            | "node_modules"
                            | "dist"
                            | "build"
                            | ".venv"
                            | "venv"
                            | "__pycache__"
                            | ".gpui-run"
                    )
                ) {
                    stack.push(path);
                }
            } else if metadata.is_file() {
                let relative = path.strip_prefix(&root)?.to_owned();
                if relative
                    .to_string_lossy()
                    .to_lowercase()
                    .replace('\\', "/")
                    .contains(&query)
                {
                    files.push(relative);
                }
            }
        }
        if visited > 100_000 {
            break;
        }
    }
    files.sort_by_key(|path| {
        (
            path.components().count(),
            path.to_string_lossy().to_lowercase(),
        )
    });
    files.truncate(100);
    Ok(files)
}
