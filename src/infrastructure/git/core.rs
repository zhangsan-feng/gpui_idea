use crate::domain::git::{GitCommit, GitDetails, GitFileChange, GitSnapshot};
use anyhow::{Context as _, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::{Output, Stdio},
    time::Duration,
};
use tokio::process::Command;

async fn output(root: &Path, args: &[&str]) -> Result<Output> {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args(["-c", "core.quotepath=false", "-c", "color.ui=false"])
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .stdin(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    tokio::time::timeout(Duration::from_secs(15), command.output())
        .await
        .context("读取 Git 信息超时")?
        .context("无法启动 Git，请检查 PATH")
}

async fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let result = output(root, args).await?;
    ensure!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    Ok(result.stdout)
}

pub(super) async fn snapshot(root: PathBuf) -> Result<GitSnapshot> {
    let discovery = output(&root, &["rev-parse", "--is-inside-work-tree"]).await?;
    if !discovery.status.success() {
        let error = String::from_utf8_lossy(&discovery.stderr);
        if error.contains("not a git repository") {
            tracing::debug!(root = %root.display(), "Project has no Git repository");
            return Ok(GitSnapshot::default());
        }
        anyhow::bail!("{}", error.trim());
    }
    if String::from_utf8_lossy(&discovery.stdout).trim() != "true" {
        return Ok(GitSnapshot::default());
    }
    let branch = run(&root, &["branch", "--show-current"]).await?;
    let branch = String::from_utf8_lossy(&branch).trim().to_owned();
    let is_detached = branch.is_empty();
    let branch = if is_detached {
        let hash = run(&root, &["rev-parse", "--short", "HEAD"]).await?;
        format!("HEAD · {}", String::from_utf8_lossy(&hash).trim())
    } else {
        branch
    };
    let mut snapshot = GitSnapshot {
        is_repository: true,
        is_detached,
        branch,
        ..Default::default()
    };
    // A log/status failure must not erase a successfully read repository/branch.
    let history = async {
        let status = run(&root, &["status", "--short", "--branch"]).await?;
        snapshot.status = String::from_utf8_lossy(&status).into_owned();
        if snapshot.status.lines().count() <= 1 {
            snapshot.status.push_str("\n工作区干净");
        }
        let head = output(&root, &["rev-parse", "--verify", "--quiet", "HEAD"]).await?;
        if !head.status.success() {
            ensure!(
                head.status.code() == Some(1),
                "{}",
                String::from_utf8_lossy(&head.stderr).trim()
            );
            return Ok::<_, anyhow::Error>(());
        }
        let commits = run(
            &root,
            &[
                "log",
                "-100",
                "--date=short",
                "--format=%H%x1f%h%x1f%s%x1f%an%x1f%ad%x1f%D",
            ],
        )
        .await?;
        for line in String::from_utf8_lossy(&commits).lines() {
            let values: Vec<_> = line.splitn(6, '\x1f').collect();
            if values.len() == 6 {
                snapshot.commits.push(GitCommit {
                    hash: values[0].into(),
                    short_hash: values[1].into(),
                    subject: values[2].into(),
                    author: values[3].into(),
                    date: values[4].into(),
                    references: values[5].into(),
                });
            }
        }
        Ok(())
    }
    .await;
    if let Err(error) = history {
        snapshot.error = Some(format!("{error:#}"));
    }
    tracing::debug!(root = %root.display(), branch = snapshot.branch, commits = snapshot.commits.len(), "Git snapshot loaded");
    Ok(snapshot)
}

pub(super) async fn details(root: PathBuf, hash: String) -> Result<GitDetails> {
    ensure!(
        matches!(hash.len(), 40 | 64) && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "提交标识无效"
    );
    let output = run(
        &root,
        &[
            "show",
            "--name-status",
            "--format=",
            "--no-renames",
            "-z",
            &hash,
        ],
    )
    .await?;
    let fields: Vec<_> = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let files = fields
        .chunks_exact(2)
        .map(|pair| GitFileChange {
            status: String::from_utf8_lossy(pair[0]).trim().into(),
            path: PathBuf::from(String::from_utf8_lossy(pair[1]).as_ref()),
        })
        .collect();
    Ok(GitDetails {
        hash,
        files,
        error: None,
    })
}
