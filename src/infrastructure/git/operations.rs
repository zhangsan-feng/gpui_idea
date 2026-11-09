use super::core::{output, run, run_input};
use crate::domain::git::GitAction;
use anyhow::{Result, ensure};
use std::path::{Path, PathBuf};

fn file_argument(path: &Path) -> Result<&str> {
    ensure!(
        !path.as_os_str().is_empty()
            && path
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_))),
        "Git 文件路径无效"
    );
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Git 路径编码无效"))
}

pub(super) async fn operate(root: PathBuf, action: GitAction) -> Result<String> {
    // Literal pathspecs keep filenames containing glob characters from affecting other files.
    let bytes = match action {
        GitAction::Ignore {
            path, directory, ..
        } => {
            return super::ignore::add(root, path, directory).await;
        }
        GitAction::Commit {
            summary,
            description,
            paths,
        } => {
            ensure!(!summary.trim().is_empty(), "请输入提交摘要");
            ensure!(!paths.is_empty(), "请选择要提交的文件");
            let mut input = Vec::new();
            for path in &paths {
                input.extend_from_slice(file_argument(path)?.as_bytes());
                input.push(0);
            }
            let message = if description.trim().is_empty() {
                summary.trim().to_owned()
            } else {
                format!("{}\n\n{}", summary.trim(), description.trim())
            };
            // Include working-tree contents for selected paths while leaving unrelated staged paths out of the commit.
            run_input(
                &root,
                &[
                    "--literal-pathspecs",
                    "add",
                    "-A",
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input.clone()),
            )
            .await?;
            run_input(
                &root,
                &[
                    "--literal-pathspecs",
                    "commit",
                    "--only",
                    "-m",
                    &message,
                    "--pathspec-from-file=-",
                    "--pathspec-file-nul",
                ],
                Some(input),
            )
            .await?
        }
        GitAction::UpdateFromRemote => run(&root, &["pull", "--ff-only"]).await?,
        GitAction::Switch(branch) => {
            validate_branch(&root, &branch).await?;
            run(&root, &["switch", &branch]).await?
        }
    };
    Ok(String::from_utf8_lossy(&bytes).trim().to_owned())
}

async fn validate_branch(root: &Path, branch: &str) -> Result<()> {
    ensure!(
        !branch.starts_with('-') && !branch.is_empty(),
        "分支名称无效"
    );
    run(root, &["check-ref-format", "--branch", branch]).await?;
    Ok(())
}

pub(super) async fn diff(
    root: PathBuf,
    path: PathBuf,
    staged: bool,
    commit: Option<String>,
    original_path: Option<PathBuf>,
) -> Result<String> {
    let file = file_argument(&path)?;
    let bytes = if let Some(hash) = commit {
        ensure!(
            matches!(hash.len(), 40 | 64) && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "提交标识无效"
        );
        let mut args = vec![
            "--literal-pathspecs",
            "show",
            "--format=",
            "--find-renames",
            "--diff-merges=first-parent",
            "--no-ext-diff",
            "--no-textconv",
            &hash,
            "--",
            file,
        ];
        if let Some(original) = &original_path {
            args.push(file_argument(original)?);
        }
        run(&root, &args).await?
    } else if staged {
        run(
            &root,
            &[
                "--literal-pathspecs",
                "diff",
                "--cached",
                "--no-ext-diff",
                "--no-textconv",
                "--",
                file,
            ],
        )
        .await?
    } else {
        let head = output(&root, &["rev-parse", "--verify", "--quiet", "HEAD"]).await?;
        let bytes = if head.status.success() {
            run(
                &root,
                &[
                    "--literal-pathspecs",
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "HEAD",
                    "--",
                    file,
                ],
            )
            .await?
        } else {
            Vec::new()
        };
        if bytes.is_empty() {
            let tracked = output(
                &root,
                &[
                    "--literal-pathspecs",
                    "ls-files",
                    "--error-unmatch",
                    "--",
                    file,
                ],
            )
            .await?;
            if (!tracked.status.success() || !head.status.success()) && root.join(&path).is_file() {
                let preview_path = root.join(&path);
                let bytes = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
                    let metadata = std::fs::metadata(&preview_path)?;
                    ensure!(metadata.len() <= 2 * 1024 * 1024, "文件超过 2 MB，暂不预览");
                    Ok(std::fs::read(preview_path)?)
                })
                .await??;
                if bytes.contains(&0) {
                    return Ok("二进制文件，暂不支持文本差异预览".into());
                }
                return Ok(format!(
                    "--- /dev/null\n+++ {file}\n{}",
                    String::from_utf8_lossy(&bytes)
                        .lines()
                        .map(|line| format!("+{line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
        bytes
    };
    ensure!(bytes.len() <= 4 * 1024 * 1024, "差异超过 4 MB，暂不预览");
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(if text.is_empty() {
        "没有文本差异（可能只有文件模式发生变化）".into()
    } else {
        text
    })
}
