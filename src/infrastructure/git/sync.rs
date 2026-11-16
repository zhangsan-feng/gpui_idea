use super::core::{output, run};
use crate::domain::git::GitDataContextSnapshot;
use anyhow::{Result, ensure};
use std::path::Path;

struct PushTarget {
    remote: String,
    branch: String,
    tracking: String,
    needs_upstream: bool,
}

async fn target(root: &Path) -> Result<Option<PushTarget>> {
    let branch = run(root, &["branch", "--show-current"]).await?;
    let branch = String::from_utf8_lossy(&branch).trim().to_owned();
    if branch.is_empty() {
        return Ok(None);
    }
    let reference = format!("refs/heads/{branch}");
    let upstream = run(
        root,
        &[
            "for-each-ref",
            "--format=%(upstream:remotename)%00%(upstream:remoteref)%00%(upstream)",
            &reference,
        ],
    )
    .await?;
    let upstream = String::from_utf8_lossy(&upstream);
    let fields: Vec<_> = upstream.trim_end().split('\0').collect();
    if fields.len() == 3 && !fields[0].is_empty() && !fields[1].is_empty() {
        return Ok(Some(PushTarget {
            remote: fields[0].into(),
            branch: fields[1].into(),
            tracking: fields[2].into(),
            needs_upstream: false,
        }));
    }
    let remotes = run(root, &["remote"]).await?;
    let remotes = String::from_utf8_lossy(&remotes);
    let remotes: Vec<_> = remotes.lines().collect();
    let remote = if remotes.contains(&"origin") {
        "origin"
    } else if remotes.len() == 1 {
        remotes[0]
    } else {
        return Ok(None);
    };
    Ok(Some(PushTarget {
        remote: remote.into(),
        branch: reference,
        tracking: format!("refs/remotes/{remote}/{branch}"),
        needs_upstream: true,
    }))
}

pub(super) async fn load_status(root: &Path, snapshot: &mut GitDataContextSnapshot) -> Result<()> {
    let Some(target) = target(root).await? else {
        return Ok(());
    };
    snapshot.push_remote = Some(target.remote);
    snapshot.needs_upstream = target.needs_upstream;
    let tracking = output(root, &["show-ref", "--verify", "--quiet", &target.tracking]).await?;
    if tracking.status.success() {
        let range = format!("{}...HEAD", target.tracking);
        let counts = run(root, &["rev-list", "--left-right", "--count", &range]).await?;
        let counts = String::from_utf8_lossy(&counts);
        let values = counts.split_whitespace().collect::<Vec<_>>();
        ensure!(values.len() == 2, "Git 同步计数格式无效");
        snapshot.behind = values[0].parse()?;
        snapshot.ahead = values[1].parse()?;
    } else {
        ensure!(tracking.status.code() == Some(1), "无法读取远程跟踪分支");
        let count = run(root, &["rev-list", "--count", "HEAD"]).await?;
        snapshot.ahead = String::from_utf8_lossy(&count).trim().parse()?;
        snapshot.needs_upstream = true;
    }
    tracing::debug!(ahead = snapshot.ahead, behind = snapshot.behind, remote = ?snapshot.push_remote, "Git push status loaded");
    Ok(())
}

pub(super) async fn push(root: &Path) -> Result<String> {
    let target = target(root)
        .await?
        .ok_or_else(|| anyhow::anyhow!("无法确定推送目标，请配置当前分支的上游远程分支"))?;
    let refspec = format!("HEAD:{}", target.branch);
    tracing::info!(
        remote = target.remote,
        branch = target.branch,
        "Git push started"
    );
    let mut args = vec!["push"];
    if target.needs_upstream {
        args.push("--set-upstream");
    }
    args.extend(["--", &target.remote, &refspec]);
    let result = output(root, &args).await?;
    let message = [
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    ]
    .iter()
    .map(|text| text.trim())
    .filter(|text| !text.is_empty())
    .collect::<Vec<_>>()
    .join("\n");
    if !result.status.success() {
        tracing::warn!(remote = target.remote, status = %result.status, "Git push failed");
        anyhow::bail!("推送失败，本地提交已保留，可以再次推送。\n{message}");
    }
    tracing::info!(remote = target.remote, "Git push succeeded");
    Ok(format!("已推送到 {}。\n{message}", target.remote))
}
