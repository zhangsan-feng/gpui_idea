use super::{LanguageServers, languages};
use crate::domain::run::RunConfigurationDataContextSnapshot;
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
    time::Duration,
};

pub(in crate::infrastructure) fn local_file_run_configuration(
    root: &Path,
    file: &Path,
) -> Result<Option<RunConfigurationDataContextSnapshot>> {
    let Some(support) = languages::ALL
        .iter()
        .find(|language| language.supports_file(file))
        .and_then(|language| language.file_run.as_ref())
    else {
        return Ok(None);
    };
    if !(support.accepts_file)(file) {
        return Ok(None);
    }
    let root = root.canonicalize()?;
    let file = file.canonicalize()?;
    anyhow::ensure!(file.starts_with(&root), "文件不在当前工作区内");
    let configuration = (support.resolve)(&root, &file)?;
    anyhow::ensure!(
        configuration
            .profiles
            .iter()
            .any(|profile| !profile.command.is_empty()),
        "此文件没有可运行入口"
    );
    Ok(Some(configuration))
}

impl LanguageServers {
    pub(crate) fn supports_file_run(&self, file: &Path) -> bool {
        languages::ALL
            .iter()
            .find(|language| language.supports_file(file))
            .and_then(|language| language.file_run.as_ref())
            .is_some_and(|support| (support.accepts_file)(file))
    }

    pub(crate) fn is_file_run_project_marker(&self, file: &Path) -> bool {
        let Some(name) = file.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        languages::ALL
            .iter()
            .any(|language| language.file_run.is_some() && language.markers.contains(&name))
    }

    pub(crate) fn has_server_file_run_support(&self, file: &Path) -> bool {
        languages::ALL
            .iter()
            .find(|language| language.supports_file(file))
            .and_then(|language| language.file_run.as_ref())
            .is_some_and(|support| (support.accepts_file)(file) && support.server.is_some())
    }

    // Outer None means unavailable/unsupported; inner None is an authoritative empty result.
    // Background capability discovery never starts or installs a language server.
    pub(crate) async fn server_file_run_configuration(
        &self,
        root: PathBuf,
        relative: PathBuf,
        synchronize: bool,
    ) -> Result<Option<Option<RunConfigurationDataContextSnapshot>>> {
        let Some(adapter) = languages::ALL
            .iter()
            .find(|adapter| adapter.supports_file(&relative))
        else {
            return Ok(None);
        };
        let Some(support) = adapter.file_run.as_ref() else {
            return Ok(None);
        };
        if !(support.accepts_file)(&relative) {
            return Ok(Some(None));
        }
        let Some(server) = support.server.as_ref() else {
            return Ok(None);
        };
        let cell = match self.session_slot(root, adapter.family, false).await {
            Ok(cell) => cell,
            Err(_) => return Ok(None),
        };
        let root = cell.root.clone();
        let checked_root = root.clone();
        let checked_relative = relative.clone();
        let file = tokio::task::spawn_blocking(move || {
            crate::infrastructure::filesystem::run_tools::checked_file(
                &checked_root,
                &checked_relative,
            )
        })
        .await??;
        let text = if synchronize {
            let file = file.clone();
            Some(tokio::task::spawn_blocking(move || std::fs::read_to_string(file)).await??)
        } else {
            None
        };
        let mut changes = cell.generation.subscribe();
        let mut session = match tokio::time::timeout(
            Duration::from_millis(if synchronize { 15000 } else { 100 }),
            cell.session.lock(),
        )
        .await
        {
            Ok(session) => session,
            Err(_) => return Ok(None),
        };
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let Some(active) = session.as_mut().filter(|session| session.run_supported()) else {
            return Ok(None);
        };
        if !synchronize && active.indexing() {
            return Ok(None);
        }
        let result = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(30), active.file_runnables(server.method, server.parameters, file, text, adapter.family)) => result.context("语言服务运行目标识别超时").and_then(|result| result),
            _ = changes.changed() => Err(anyhow::anyhow!("运行目标识别让位于编辑器请求")),
        };
        if result.is_err() {
            active.cancel_current_request().await;
        }
        let response = result?;
        let configuration = (server.parse)(&root, &response)?;
        tracing::debug!(file = %relative.display(), runnable = configuration.is_some(), family = adapter.family, "LSP file run target resolved");
        Ok(Some(configuration))
    }
}
