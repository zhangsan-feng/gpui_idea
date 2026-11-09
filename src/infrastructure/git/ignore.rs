use super::core::run;
use anyhow::{Context as _, Result, ensure};
use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Component, PathBuf},
};

pub(super) async fn add(root: PathBuf, path: PathBuf, directory: bool) -> Result<String> {
    ensure!(
        !path.as_os_str().is_empty()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_))),
        "忽略路径必须位于仓库内，且不能是仓库根目录"
    );
    let literal = path.to_str().context("忽略路径编码无效")?;
    ensure!(
        !literal.contains(['\r', '\n', '\0']),
        "忽略路径包含不支持的换行或空字符"
    );
    let tracked = !run(
        &root,
        &["--literal-pathspecs", "ls-files", "-z", "--", literal],
    )
    .await?
    .is_empty();
    // Anchor to the repository root and escape glob metacharacters so choosing
    // one path cannot accidentally ignore similarly named files elsewhere.
    let mut pattern = String::from("/");
    for (index, part) in path.components().enumerate() {
        if index > 0 {
            pattern.push('/');
        }
        for ch in part
            .as_os_str()
            .to_str()
            .context("忽略路径编码无效")?
            .chars()
        {
            if matches!(ch, '\\' | '*' | '?' | '[' | ']' | '#' | '!' | ' ') {
                pattern.push('\\');
            }
            pattern.push(ch);
        }
    }
    if directory {
        pattern.push('/');
    }
    let log_root = root.clone();
    let written = pattern.clone();
    let added = tokio::task::spawn_blocking(move || -> Result<bool> {
        let target = root.join(".gitignore");
        let existing = match fs::symlink_metadata(&target) {
            Ok(metadata) => {
                ensure!(
                    metadata.file_type().is_file(),
                    ".gitignore 必须是普通文件，不能是链接或目录"
                );
                fs::read_to_string(&target).context("读取 .gitignore 失败")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error).context("读取 .gitignore 属性失败"),
        };
        // Append again if a later negation could override an earlier rule.
        let mut effective = false;
        for line in existing.trim_start_matches('\u{feff}').lines() {
            if line == written {
                effective = true;
            } else if line.starts_with('!') {
                effective = false;
            }
        }
        if effective {
            return Ok(false);
        }
        let newline = if existing.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let mut suffix = String::new();
        if !existing.is_empty() && !existing.ends_with('\n') {
            suffix.push_str(newline);
        }
        suffix.push_str(&written);
        suffix.push_str(newline);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target)
            .context("打开 .gitignore 失败")?;
        file.write_all(suffix.as_bytes())
            .context("写入 .gitignore 失败")?;
        Ok(true)
    })
    .await
    .context("写入忽略规则的后台任务失败")??;
    tracing::info!(root = %log_root.display(), %pattern, directory, tracked, added, "Git ignore rule updated");
    let message = if added {
        "已加入 .gitignore"
    } else {
        ".gitignore 已有该规则"
    };
    Ok(format!(
        "{message}：{pattern}{}",
        if tracked {
            "；已跟踪的文件仍会显示改动，该规则用于忽略未跟踪文件"
        } else {
            ""
        }
    ))
}
