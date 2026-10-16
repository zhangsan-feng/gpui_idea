use crate::{
    application::APPLICATION,
    data_context::{DATA_CONTEXT, WorkspaceStartup},
    domain::history::OpenKind,
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Context as _, Result};

pub(super) fn initialize() -> Result<()> {
    INFRASTRUCTURE.initialize()?;
    std::sync::LazyLock::force(&DATA_CONTEXT);
    std::sync::LazyLock::force(&APPLICATION);
    Ok(())
}

pub(super) async fn prepare_startup() -> Result<()> {
    let mut arguments = std::env::args_os().skip(1).collect::<Vec<_>>().into_iter();
    let mut requested = None;
    while let Some(argument) = arguments.next() {
        if argument == "--project" {
            let path =
                std::path::PathBuf::from(arguments.next().context("--project 缺少项目目录")?);
            requested = Some(INFRASTRUCTURE.projects.validate(path).await?);
        }
    }
    let restored = if let Some(root) = &requested {
        APPLICATION.history.restore_project(root.clone()).await
    } else {
        APPLICATION.history.restore_session().await
    };
    let (session, notice) = match restored {
        Ok(session) => (session, None),
        Err(error) => {
            tracing::error!(error = %error, "workspace session restore failed; using default project");
            (None, Some(format!("无法恢复上次工作区：{error:#}")))
        }
    };
    let root = session
        .as_ref()
        .map(|session| session.project_root.clone())
        .or(requested)
        .unwrap_or_else(|| crate::infrastructure::paths::get().root.clone());
    let initial_file = INFRASTRUCTURE
        .filesystem
        .list_directory(root.clone(), std::path::PathBuf::new())
        .await
        .ok()
        .and_then(|entries| {
            entries.into_iter().find(|entry| {
                !entry.is_directory
                    && !entry.is_link
                    && matches!(entry.name.as_str(), "Cargo.toml" | "README.md")
            })
        })
        .map(|entry| entry.relative_path);
    DATA_CONTEXT.set_startup(WorkspaceStartup {
        project_root: root.clone(),
        session,
        notice,
        initial_file,
    })?;
    if let Err(error) = APPLICATION.project.initialize_run_configuration().await {
        tracing::warn!(error = %format!("{error:#}"), "initial project run configuration unavailable");
    }
    if let Err(error) = APPLICATION
        .history
        .record_opened(OpenKind::Project, root.clone())
        .await
    {
        tracing::warn!(error = %error, "project opened but history could not be persisted");
    }
    tracing::info!(root = %root.display(), "shared workspace state initialized");
    APPLICATION.file_watch.start().await?;
    Ok(())
}
