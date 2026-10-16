mod compatible;
mod core;
mod download;
mod environment;
mod http;
mod native;
mod node;
mod update;

use crate::domain::language_server::LanguageServerStatus;
use anyhow::Result;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::watch;

pub(super) use download::download_server;
pub(super) use environment::{go_server, java_requirement, java_runtime};
pub(super) use native::rust_server;
pub(super) use node::node_server;
pub(super) use update::refresh;

pub(super) struct InstallContext {
    status: watch::Sender<LanguageServerStatus>,
    stopping: Arc<AtomicBool>,
    inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    server_id: u64,
    selection: std::sync::Mutex<Option<String>>,
}

impl InstallContext {
    pub(in crate::infrastructure::language_server) fn before_launch(&self) -> Result<()> {
        self.check()?;
        self.status.send_replace(LanguageServerStatus::Starting);
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == self.server_id)
            {
                server.phase = crate::domain::language_server::LanguageServerPhase::Starting;
                server.detail = "正在启动并初始化语言服务".into();
            }
        });
        Ok(())
    }
    pub(super) fn new(
        status: watch::Sender<LanguageServerStatus>,
        stopping: Arc<AtomicBool>,
        inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
        server_id: u64,
    ) -> Self {
        Self {
            status,
            stopping,
            inventory,
            server_id,
            selection: std::sync::Mutex::new(None),
        }
    }

    pub(in crate::infrastructure::language_server) fn launcher(
        &self,
        path: PathBuf,
        version: Option<String>,
    ) {
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == self.server_id)
            {
                server.executable = Some(path.clone());
                server.version = version.clone();
            }
        });
        tracing::info!(server_id = self.server_id, executable = %path.display(), "LSP executable selected");
    }

    fn target(&self, path: PathBuf) {
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == self.server_id)
            {
                server.install_directory = Some(path.clone());
            }
        });
    }

    fn selected_version(&self) -> Option<String> {
        self.selection
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn selection(&self, detail: String) -> Result<()> {
        *self
            .selection
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(detail.clone());
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == self.server_id)
            {
                server.selection_note = Some(detail.clone());
            }
        });
        tracing::info!(%detail, "Selected compatible LSP release");
        self.progress(detail)
    }

    fn check(&self) -> Result<()> {
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务准备已取消");
        Ok(())
    }

    fn progress(&self, detail: impl Into<String>) -> Result<()> {
        self.check()?;
        let detail = detail.into();
        tracing::debug!(%detail, "Preparing managed language server");
        self.status
            .send_replace(LanguageServerStatus::Preparing(detail.clone()));
        self.inventory.send_modify(|servers| {
            if let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == self.server_id)
            {
                server.phase = crate::domain::language_server::LanguageServerPhase::Installing;
                server.detail = detail.clone();
            }
        });
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Installed {
    entry: PathBuf,
    version: String,
    source: String,
    #[serde(default)]
    selection: Option<String>,
}

#[derive(Clone)]
pub(super) struct Binary {
    pub(super) path: PathBuf,
    pub(super) version: String,
    selection: Option<String>,
}
