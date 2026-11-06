use super::*;
use crate::domain::git::{GitAction, GitDiffSnapshot};

impl ProjectService {
    pub(in crate::application::project) async fn git_action_core(
        &self,
        action: GitAction,
    ) -> Result<String> {
        let _guard = self.git_operations.lock().await;
        let snapshot = DATA_CONTEXT.project.git();
        anyhow::ensure!(snapshot.is_repository, "当前项目不是 Git 仓库");
        tracing::info!(root = %snapshot.root.display(), "Git operation started");
        let result = INFRASTRUCTURE
            .git
            .operate(snapshot.root.clone(), action)
            .await;
        if let Err(error) = &result {
            tracing::warn!(%error, "Git operation failed");
        }
        // Refresh even after an error: hooks or network operations may have changed state.
        let refresh = self.git_status_core().await;
        if let Err(error) = refresh {
            tracing::warn!(%error, "Git refresh after operation failed");
        }
        result
    }

    pub(in crate::application::project) async fn git_diff_core(
        &self,
        key: String,
        path: PathBuf,
        staged: bool,
        commit: Option<String>,
    ) -> Result<()> {
        let _guard = self.git_operations.lock().await;
        let repository = DATA_CONTEXT.project.git();
        anyhow::ensure!(repository.is_repository, "当前项目不是 Git 仓库");
        let original_path = commit.as_ref().and_then(|hash| {
            DATA_CONTEXT
                .project
                .git_details(hash)
                .files
                .iter()
                .find(|file| file.path == path)
                .and_then(|file| file.original_path.clone())
        });
        let result = INFRASTRUCTURE
            .git
            .diff(repository.root.clone(), path, staged, commit, original_path)
            .await;
        let snapshot = match &result {
            Ok(text) => GitDiffSnapshot {
                key,
                lines: text.lines().map(str::to_owned).collect(),
                error: None,
            },
            Err(error) => GitDiffSnapshot {
                key,
                error: Some(format!("{error:#}")),
                ..Default::default()
            },
        };
        DATA_CONTEXT.project.set_git_diff(snapshot);
        result.map(|_| ())
    }
}
