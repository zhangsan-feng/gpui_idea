use super::core::run;
use crate::domain::git::GitCreateRequest;
use anyhow::{Context as _, Result, ensure};
use std::path::{Path, PathBuf};

pub(super) async fn create(root: PathBuf, request: GitCreateRequest) -> Result<String> {
    match request {
        GitCreateRequest::Branch { name, remote } => {
            ensure!(
                !name.is_empty() && !name.starts_with('-'),
                "请输入有效的分支名称"
            );
            run(&root, &["check-ref-format", "--branch", &name]).await?;
            // Require a commit before creating either kind of branch.
            run(&root, &["rev-parse", "--verify", "HEAD"])
                .await
                .context("当前仓库还没有提交，请先提交后再创建分支")?;
            if let Some(remote) = remote {
                require_remote(&root, &remote, true).await?;
                let reference = format!("refs/heads/{name}");
                let existing = run(&root, &["ls-remote", "--heads", &remote, &reference]).await?;
                ensure!(existing.is_empty(), "远程分支已存在，请更换名称");
                // An empty lease also protects against a branch created after the check.
                run(
                    &root,
                    &[
                        "push",
                        &format!("--force-with-lease={reference}:"),
                        &remote,
                        &format!("HEAD:{reference}"),
                    ],
                )
                .await?;
                Ok(format!(
                    "已创建远程分支 {remote}/{name}，当前本地分支保持不变"
                ))
            } else {
                run(&root, &["switch", "-c", &name]).await?;
                Ok(format!("已创建并切换到本地分支 {name}"))
            }
        }
    }
}

fn valid_remote(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

async fn require_remote(root: &Path, remote: &str, exists: bool) -> Result<()> {
    ensure!(valid_remote(remote), "远端名称无效");
    let remotes = run(root, &["remote"]).await?;
    let found = String::from_utf8_lossy(&remotes)
        .lines()
        .any(|name| name == remote);
    ensure!(
        found == exists,
        "{}",
        if exists {
            "远端不存在，请先配置远程仓库"
        } else {
            "远端名称已存在，请更换名称"
        }
    );
    Ok(())
}
