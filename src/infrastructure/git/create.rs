use super::core::{output, run};
use crate::domain::git::GitCreateRequest;
use anyhow::{Context as _, Result, ensure};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

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
        GitCreateRequest::Repository {
            name,
            organization,
            description,
            private,
            remote,
            token,
        } => {
            ensure!(
                valid_name(&name),
                "仓库名称仅支持字母、数字、点、下划线和连字符，且不能仅为点"
            );
            ensure!(
                organization.is_empty()
                    || organization
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
                "组织名称无效"
            );
            ensure!(!token.trim().is_empty(), "请输入 GitHub 访问令牌");
            ensure!(valid_remote(&remote), "远端名称无效");
            run(
                &root,
                &["check-ref-format", &format!("refs/remotes/{remote}/probe")],
            )
            .await
            .context("远端名称无效")?;
            let repository = output(&root, &["rev-parse", "--is-inside-work-tree"]).await?;
            let initialized = repository.status.success();
            if initialized {
                require_remote(&root, &remote, false).await?;
            } else {
                ensure!(valid_remote(&remote), "远端名称无效");
                ensure!(
                    String::from_utf8_lossy(&repository.stderr).contains("not a git repository"),
                    "无法读取本地 Git 仓库，请检查目录权限和 Git 配置"
                );
            }
            let (url, clone_url) =
                create_repository(name, organization, description, private, token).await?;
            let attach = async {
                if !initialized {
                    run(&root, &["init"]).await?;
                }
                run(&root, &["remote", "add", &remote, &clone_url]).await?;
                Ok::<_, anyhow::Error>(())
            }
            .await;
            attach.with_context(|| {
                format!(
                    "GitHub 仓库已创建：{url}；关联本地远端失败，请手动添加远端，无需重复创建仓库"
                )
            })?;
            Ok(format!(
                "GitHub 仓库已创建：{url}\n已关联远端 {remote}，尚未推送本地提交。"
            ))
        }
    }
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 100
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
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

async fn create_repository(
    name: String,
    organization: String,
    description: String,
    private: bool,
    token: String,
) -> Result<(String, String)> {
    let endpoint = if organization.is_empty() {
        "https://api.github.com/user/repos".to_owned()
    } else {
        format!("https://api.github.com/orgs/{organization}/repos")
    };
    let client = reqwest::Client::builder()
        .user_agent("gpui-idea")
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
        .build()?;
    let response = client.post(endpoint)
        .bearer_auth(token.trim())
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&serde_json::json!({"name": name, "description": description, "private": private, "auto_init": false}))
        .send().await.context("GitHub 创建请求失败；重试前请先确认网站上是否已创建仓库")?;
    let status = response.status();
    // Keep server payloads and credentials out of error logs.
    ensure!(
        status.is_success(),
        "GitHub 创建仓库失败（HTTP {}）：{}",
        status.as_u16(),
        match status.as_u16() {
            401 => "令牌无效或已过期",
            403 => "令牌权限不足、组织限制或 API 限流",
            404 => "组织不存在或当前账号无权访问",
            422 => "仓库名称已存在或参数不符合 GitHub 要求",
            429 => "请求已被限流，请稍后再操作",
            _ => "请检查网络和 GitHub 服务状态，重试前确认仓库是否已创建",
        }
    );
    let value: serde_json::Value = response
        .json()
        .await
        .context("GitHub 返回内容无法解析，请到网站确认仓库已创建")?;
    let url = value["html_url"]
        .as_str()
        .context("GitHub 未返回仓库页面地址，请到网站确认创建结果")?;
    let clone_url = value["clone_url"]
        .as_str()
        .context("GitHub 未返回克隆地址，请到网站确认创建结果")?;
    ensure!(
        url.starts_with("https://github.com/") && clone_url.starts_with("https://github.com/"),
        "GitHub 返回的仓库地址无效"
    );
    tracing::info!(private, "GitHub repository created via API");
    Ok((url.to_owned(), clone_url.to_owned()))
}
