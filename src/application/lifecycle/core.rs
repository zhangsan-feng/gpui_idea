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
    APPLICATION.settings.load().await?;
    let mut arguments = std::env::args_os().skip(1).collect::<Vec<_>>().into_iter();
    let mut requested = None;
    let mut requested_file = None;
    while let Some(argument) = arguments.next() {
        if argument == "--project" {
            let path =
                std::path::PathBuf::from(arguments.next().context("--project 缺少项目目录")?);
            requested = Some(INFRASTRUCTURE.projects.validate(path).await?);
        } else if argument == "--file" {
            let path = std::path::PathBuf::from(arguments.next().context("--file 缺少文件路径")?);
            let path = INFRASTRUCTURE
                .history
                .validate_target(OpenKind::File, path)
                .await?;
            requested_file = Some(path);
        }
    }
    if let Some(path) = requested_file {
        let notice = match APPLICATION.history.load().await {
            Ok(()) => None,
            Err(error) => {
                tracing::error!(error = %format!("{error:#}"), "standalone file project history load failed");
                Some(format!("无法读取最近项目：{error:#}"))
            }
        };
        tracing::info!(file = %path.display(), "standalone file workspace initialized; project services not started");
        DATA_CONTEXT.set_startup(WorkspaceStartup {
            project_root: std::path::PathBuf::new(),
            session: None,
            notice,
            initial_file: Some(path),
        })?;
        return Ok(());
    }
    let restored = if let Some(root) = &requested {
        APPLICATION.history.restore_project(root.clone()).await
    } else {
        APPLICATION.history.restore_session().await
    };
    let (session, notice) = match restored {
        Ok(session) => (session, None),
        Err(error) => {
            tracing::error!(error = %error, "workspace session restore failed; opening empty workspace");
            (None, Some(format!("无法恢复上次工作区：{error:#}")))
        }
    };
    let root = session
        .as_ref()
        .map(|session| session.project_root.clone())
        .or(requested)
        .unwrap_or_default();
    let initial_file = if root.as_os_str().is_empty() {
        None
    } else {
        INFRASTRUCTURE
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
            .map(|entry| entry.relative_path)
    };
    DATA_CONTEXT.set_startup(WorkspaceStartup {
        project_root: root.clone(),
        session,
        notice,
        initial_file,
    })?;
    if !DATA_CONTEXT.has_project() {
        tracing::info!("empty workspace initialized; project services not started");
        return Ok(());
    }
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
