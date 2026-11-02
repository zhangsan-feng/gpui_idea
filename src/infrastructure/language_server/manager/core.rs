use super::super::{LanguageServers, SessionSlot, core, languages};
use crate::domain::language_server::LanguageServerStatus;
use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

impl LanguageServers {
    // Workspace identity is independent of the configuration nearest to a file.
    // Every entry point uses this key, including lookups that must not start a server.
    pub(in crate::infrastructure::language_server) async fn session_slot(
        &self,
        workspace: PathBuf,
        family: &'static str,
        create: bool,
    ) -> Result<Arc<SessionSlot>> {
        let workspace = tokio::task::spawn_blocking(move || {
            std::fs::canonicalize(&workspace)
                .with_context(|| format!("无法定位语言服务工作区 {}", workspace.display()))
        })
        .await??;
        let mut sessions = self.sessions.lock().await;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let key = (workspace.clone(), family);
        if let Some(slot) = sessions.get(&key) {
            tracing::debug!(workspace = %workspace.display(), family, "Reusing workspace LSP session");
            return Ok(slot.clone());
        }
        anyhow::ensure!(create, "语言服务尚未准备好");
        languages::get(family)?;
        let slot = Arc::new(SessionSlot::new(workspace.clone(), self.status.session()));
        sessions.insert(key, slot.clone());
        tracing::info!(workspace = %workspace.display(), family, "Registered workspace LSP session");
        Ok(slot)
    }

    pub(in crate::infrastructure::language_server) fn document_family(
        path: &std::path::Path,
        language: &str,
    ) -> Result<&'static str> {
        let adapter = languages::ALL
            .iter()
            .find(|adapter| adapter.supports_file(path))
            .context("文件类型尚未接入语言服务")?;
        anyhow::ensure!(
            Some(adapter.family) == crate::domain::language_server::family(language),
            "文件类型与语言服务不匹配"
        );
        Ok(adapter.family)
    }

    async fn start_session(
        &self,
        root: &std::path::Path,
        family: &str,
        status: super::super::status::SessionStatus,
    ) -> Result<core::Session> {
        status.send_replace(LanguageServerStatus::Starting);
        match core::Session::start(
            root,
            family,
            status.clone(),
            self.inventory.clone(),
            self.stopping.clone(),
        )
        .await
        {
            Ok(session) => Ok(session),
            Err(error) => {
                tracing::warn!(%error, "Language server startup failed");
                status.send_replace(LanguageServerStatus::Error(format!("{error:#}")));
                Err(error)
            }
        }
    }

    pub(in crate::infrastructure::language_server) async fn start_slot(
        &self,
        family: &str,
        cell: &SessionSlot,
    ) -> Result<core::Session> {
        let cached = cell
            .failure
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some((at, message)) = cached {
            anyhow::ensure!(at.elapsed() >= Duration::from_secs(30), "{message}");
        }
        tracing::info!(workspace = %cell.root.display(), family, "Starting workspace LSP session");
        let result = self
            .start_session(&cell.root, family, cell.status.clone())
            .await;
        *cell
            .failure
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = result
            .as_ref()
            .err()
            .map(|error| (std::time::Instant::now(), format!("{error:#}")));
        result
    }
}
