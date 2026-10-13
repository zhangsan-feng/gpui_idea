use crate::{
    application::APPLICATION,
    data_context::{DATA_CONTEXT, WorkspaceStartup},
    domain::history::OpenKind,
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

pub(super) fn initialize() -> Result<()> {
    INFRASTRUCTURE.initialize()?;
    std::sync::LazyLock::force(&DATA_CONTEXT);
    std::sync::LazyLock::force(&APPLICATION);
    Ok(())
}

pub(super) async fn prepare_startup() -> Result<()> {
    let (session, notice) = match APPLICATION.history.restore_session().await {
        Ok(session) => (session, None),
        Err(error) => {
            tracing::error!(error = %error, "workspace session restore failed; using default project");
            (None, Some(format!("无法恢复上次工作区：{error:#}")))
        }
    };
    let root = session
        .as_ref()
        .map(|session| session.project_root.clone())
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into());
    DATA_CONTEXT.set_startup(WorkspaceStartup {
        project_root: root.clone(),
        session,
        notice,
    })?;
    if let Err(error) = APPLICATION
        .history
        .record_opened(OpenKind::Project, root.clone())
        .await
    {
        tracing::warn!(error = %error, "project opened but history could not be persisted");
    }
    tracing::info!(root = %root.display(), "shared workspace state initialized");
    Ok(())
}
