use crate::domain::git::{GitCommit, GitDetails, GitFileChange, GitSnapshot};
use anyhow::{Context, Result, ensure};
use std::{path::PathBuf, process::Command};
fn run_git(root: &std::path::Path, args: &[&str]) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command
        .current_dir(root)
        .args(["-c", "core.quotepath=false"])
        .args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let result = command.output().context("无法启动 Git")?;
    ensure!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    Ok(result.stdout)
}
pub(in crate::infrastructure::filesystem) fn git_log(root: PathBuf) -> Result<GitSnapshot> {
    let status = git_status(root.clone())?;
    let branch = String::from_utf8_lossy(&run_git(&root, &["branch", "--show-current"])?)
        .trim()
        .to_owned();
    let output = run_git(
        &root,
        &[
            "log",
            "-100",
            "--date=short",
            "--format=%H%x1f%h%x1f%s%x1f%an%x1f%ad%x1f%D",
        ],
    );
    let mut commits = Vec::new();
    match output {
        Ok(output) => {
            for line in String::from_utf8_lossy(&output).lines() {
                let values: Vec<_> = line.splitn(6, '\x1f').collect();
                if values.len() == 6 {
                    commits.push(GitCommit {
                        hash: values[0].into(),
                        short_hash: values[1].into(),
                        subject: values[2].into(),
                        author: values[3].into(),
                        date: values[4].into(),
                        references: values[5].into(),
                    });
                }
            }
        }
        Err(error) => {
            if !status.contains("No commits yet") {
                return Err(error);
            }
        }
    }
    Ok(GitSnapshot {
        branch: if branch.is_empty() {
            "Detached HEAD".into()
        } else {
            branch
        },
        commits,
        status,
        error: None,
    })
}
pub(in crate::infrastructure::filesystem) fn git_details(
    root: PathBuf,
    hash: String,
) -> Result<GitDetails> {
    ensure!(
        matches!(hash.len(), 40 | 64) && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "提交标识无效"
    );
    let output = run_git(
        &root,
        &[
            "show",
            "--name-status",
            "--format=",
            "--no-renames",
            "-z",
            &hash,
        ],
    )?;
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
pub(in crate::infrastructure::filesystem) fn git_status(root: PathBuf) -> Result<String> {
    let mut command = Command::new("git");
    command.current_dir(root).args([
        "-c",
        "core.quotepath=false",
        "status",
        "--short",
        "--branch",
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let result = command.output().context("无法启动 Git")?;
    ensure!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    let text = String::from_utf8_lossy(&result.stdout).into_owned();
    Ok(if text.lines().count() <= 1 {
        format!("{text}\n工作区干净")
    } else {
        text
    })
}
