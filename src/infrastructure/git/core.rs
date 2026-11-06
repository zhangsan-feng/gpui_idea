use crate::domain::git::{
    GitCommit, GitDataContextSnapshot, GitDetailsDataContextSnapshot, GitFileChange,
};
use anyhow::{Context as _, Result, ensure};
use std::{
    path::{Path, PathBuf},
    process::{Output, Stdio},
    time::Duration,
};
use tokio::{io::AsyncWriteExt, process::Command};

pub(super) async fn output(root: &Path, args: &[&str]) -> Result<Output> {
    output_with_input(root, args, None).await
}

async fn output_with_input(root: &Path, args: &[&str], input: Option<Vec<u8>>) -> Result<Output> {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args(["-c", "core.quotepath=false", "-c", "color.ui=false"])
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000);
    let operation = async move {
        let mut child = command.spawn()?;
        if let Some(input) = input {
            let mut stdin = child.stdin.take().context("无法写入 Git 输入")?;
            let write = async move {
                stdin.write_all(&input).await?;
                drop(stdin);
                Ok::<(), std::io::Error>(())
            };
            let (written, output) = tokio::join!(write, child.wait_with_output());
            let output = output?;
            if output.status.success() {
                written?;
            }
            Ok::<Output, anyhow::Error>(output)
        } else {
            Ok(child.wait_with_output().await?)
        }
    };
    tokio::time::timeout(Duration::from_secs(120), operation)
        .await
        .context("读取 Git 信息超时")?
        .context("无法启动 Git，请检查 PATH")
}

pub(super) async fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    run_input(root, args, None).await
}

pub(super) async fn run_input(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
) -> Result<Vec<u8>> {
    let result = output_with_input(root, args, input).await?;
    ensure!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    Ok(result.stdout)
}

pub(super) async fn snapshot(root: PathBuf) -> Result<GitDataContextSnapshot> {
    let discovery = output(&root, &["rev-parse", "--is-inside-work-tree"]).await?;
    if !discovery.status.success() {
        let error = String::from_utf8_lossy(&discovery.stderr);
        if error.contains("not a git repository") {
            tracing::debug!(root = %root.display(), "Project has no Git repository");
            return Ok(GitDataContextSnapshot::default());
        }
        anyhow::bail!("{}", error.trim());
    }
    if String::from_utf8_lossy(&discovery.stdout).trim() != "true" {
        return Ok(GitDataContextSnapshot::default());
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
    let mut snapshot = GitDataContextSnapshot {
        root: PathBuf::from(
            String::from_utf8_lossy(&run(&root, &["rev-parse", "--show-toplevel"]).await?).trim(),
        ),
        is_repository: true,
        is_detached,
        branch,
        ..Default::default()
    };
    // A log/status failure must not erase a successfully read repository/branch.
    let history = async {
        let changes = run(
            &root,
            &[
                "status",
                "--porcelain=v1",
                "--no-renames",
                "-z",
                "--untracked-files=all",
            ],
        )
        .await?;
        let mut records = changes
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty());
        while let Some(record) = records.next() {
            if record.len() < 4 {
                continue;
            }
            snapshot.changes.push(crate::domain::git::GitWorkingChange {
                index: record[0] as char,
                worktree: record[1] as char,
                path: PathBuf::from(String::from_utf8_lossy(&record[3..]).as_ref()),
            });
            if record[0] == b'R' || record[0] == b'C' || record[1] == b'R' || record[1] == b'C' {
                records.next();
            }
        }
        let branches = run(
            &root,
            &["for-each-ref", "--format=%(refname:short)", "refs/heads/"],
        )
        .await?;
        snapshot.branches = String::from_utf8_lossy(&branches)
            .lines()
            .map(str::to_owned)
            .collect();
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

pub(super) async fn details(root: PathBuf, hash: String) -> Result<GitDetailsDataContextSnapshot> {
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
            "--find-renames",
            "--diff-merges=first-parent",
            "-z",
            &hash,
        ],
    )
    .await?;
    let fields: Vec<_> = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut files = Vec::new();
    let mut fields = fields.into_iter();
    while let Some(status) = fields.next() {
        let status = String::from_utf8_lossy(status).trim().to_owned();
        let Some(first) = fields.next() else {
            anyhow::bail!("Git 历史文件列表格式无效");
        };
        let first = PathBuf::from(String::from_utf8_lossy(first).as_ref());
        let (original_path, path) = if status.starts_with('R') || status.starts_with('C') {
            let target = fields
                .next()
                .ok_or_else(|| anyhow::anyhow!("Git 重命名信息不完整"))?;
            (
                Some(first),
                PathBuf::from(String::from_utf8_lossy(target).as_ref()),
            )
        } else {
            (None, first)
        };
        files.push(GitFileChange {
            status,
            path,
            original_path,
        });
    }
    let message = run(&root, &["show", "-s", "--format=%B", &hash]).await?;
    let message = String::from_utf8_lossy(&message).trim_end().to_owned();
    Ok(GitDetailsDataContextSnapshot {
        message,
        hash,
        files,
        error: None,
    })
}
