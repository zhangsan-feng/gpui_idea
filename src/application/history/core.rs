use super::HistoryService;
use crate::domain::history::{OpenKind, WorkspaceSession};
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail};
use std::path::{Component, PathBuf};

impl HistoryService {
    pub(super) async fn forget_project_core(&self, project: PathBuf) -> Result<()> {
        let _guard = self.operations.lock().await;
        let history = INFRASTRUCTURE.history.forget_project(project).await?;
        DATA_CONTEXT.history.set_history(history);
        Ok(())
    }
    pub(super) async fn restore_project_core(
        &self,
        root: PathBuf,
    ) -> Result<Option<WorkspaceSession>> {
        self.load().await?;
        let history = DATA_CONTEXT.history.snapshot();
        let normalize = |path: &std::path::Path| {
            path.to_string_lossy()
                .trim_start_matches("\\\\?\\")
                .to_lowercase()
        };
        let session = history
            .sessions
            .iter()
            .find(|session| normalize(&session.project_root) == normalize(&root))
            .or_else(|| {
                history
                    .session
                    .as_ref()
                    .filter(|session| normalize(&session.project_root) == normalize(&root))
            })
            .cloned();
        if let Some(session) = &session {
            validate_session(session)?;
        }
        Ok(session)
    }
    pub(super) async fn load_core(&self) -> Result<()> {
        let _guard = self.operations.lock().await;
        let history = INFRASTRUCTURE.history.load().await?;
        DATA_CONTEXT.history.set_history(history);
        Ok(())
    }

    pub(super) async fn record_core(&self, kind: OpenKind, path: PathBuf) -> Result<()> {
        if !path.is_absolute() {
            bail!("历史记录路径必须是绝对路径");
        }
        let _guard = self.operations.lock().await;
        let history = INFRASTRUCTURE.history.record(kind, path.clone()).await?;
        DATA_CONTEXT.history.set_history(history);
        tracing::debug!(?kind, path = %path.display(), "opened path recorded in persistent history");
        Ok(())
    }

    pub(super) async fn save_core(&self, session: WorkspaceSession) -> Result<()> {
        validate_session(&session)?;
        let _guard = self.operations.lock().await;
        let revision = session.revision;
        let history = INFRASTRUCTURE.history.save_session(session).await?;
        DATA_CONTEXT.history.set_history(history);
        tracing::debug!(revision, "workspace session persisted");
        Ok(())
    }

    pub(super) async fn restore_core(&self) -> Result<Option<WorkspaceSession>> {
        self.load().await?;
        let Some(session) = DATA_CONTEXT.history.snapshot().session.clone() else {
            return Ok(None);
        };
        validate_session(&session)?;
        // Missing files remain as error tabs; an unavailable project falls back to the default workspace.
        INFRASTRUCTURE
            .history
            .validate_target(OpenKind::Project, session.project_root.clone())
            .await?;
        tracing::info!(project = %session.project_root.display(), files = session.open_files.len(), directories = session.expanded_directories.len(), "last workspace session loaded");
        Ok(Some(session))
    }
}

fn validate_session(session: &WorkspaceSession) -> Result<()> {
    if !session.project_root.is_absolute()
        || session.open_files.iter().any(|path| !path.is_absolute())
    {
        bail!("工作区状态必须使用绝对路径");
    }
    if session
        .active_file
        .as_ref()
        .is_some_and(|active| !session.open_files.contains(active))
    {
        bail!("当前文件必须属于打开的标签");
    }
    if session.expanded_directories.iter().any(|path| {
        path.is_absolute()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
    }) {
        bail!("展开目录必须位于项目内");
    }
    if session.file_views.iter().any(|view| {
        !view.path.is_absolute() || !view.scroll_x.is_finite() || !view.scroll_y.is_finite()
    }) || session
        .project_width
        .is_some_and(|size| !size.is_finite() || size <= 0.)
        || session
            .run_height
            .is_some_and(|size| !size.is_finite() || size <= 0.)
    {
        bail!("工作区布局状态无效");
    }
    if session.window_view.is_some_and(|view| {
        !view.width.is_finite() || !view.height.is_finite() || view.width <= 0. || view.height <= 0.
    }) {
        bail!("窗口尺寸状态无效");
    }
    Ok(())
}
