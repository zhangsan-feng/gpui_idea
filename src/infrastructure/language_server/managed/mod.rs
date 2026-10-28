mod cleanup;
pub(in crate::infrastructure::language_server) mod compatible;
pub(in crate::infrastructure::language_server) mod core;
pub(in crate::infrastructure::language_server) mod download;
pub(in crate::infrastructure::language_server) mod http;
pub(in crate::infrastructure::language_server) mod management;
mod network;
pub(in crate::infrastructure::language_server) mod node;
pub(in crate::infrastructure::language_server) mod update;

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
pub(super) use node::node_server;
pub(super) use update::refresh;

pub(super) struct InstallContext {
    status: crate::infrastructure::language_server::status::SessionStatus,
    stopping: Arc<AtomicBool>,
    inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    server_id: u64,
    selection: std::sync::Mutex<Option<String>>,
    force_update: bool,
    pub(in crate::infrastructure::language_server) requested_release: Option<String>,
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
                server.install_progress = None;
                server.detail = "正在启动并初始化语言服务".into();
            }
        });
        Ok(())
    }
    pub(super) fn new(
        status: crate::infrastructure::language_server::status::SessionStatus,
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
            force_update: false,
            requested_release: None,
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

    pub(in crate::infrastructure::language_server) fn selection(
        &self,
        detail: String,
    ) -> Result<()> {
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

    pub(in crate::infrastructure::language_server) fn check(&self) -> Result<()> {
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务准备已取消");
        Ok(())
    }

    pub(in crate::infrastructure::language_server) fn progress(
        &self,
        detail: impl Into<String>,
    ) -> Result<()> {
        self.publish_progress(detail.into(), None)
    }

    fn download_progress(&self, received_bytes: u64, total_bytes: Option<u64>) -> Result<()> {
        let total_bytes = total_bytes.filter(|total| *total > 0);
        let received = received_bytes as f64 / 1048576.0;
        let detail = match total_bytes {
            Some(total) => format!(
                "正在下载 {:.0}% · {received:.1} / {:.1} MiB",
                (received_bytes as f64 / total as f64 * 100.).min(100.),
                total as f64 / 1048576.0
            ),
            None => format!("正在下载 · 已接收 {received:.1} MiB"),
        };
        self.publish_progress(
            detail,
            Some(crate::domain::language_server::InstallProgress {
                received_bytes,
                total_bytes,
            }),
        )
    }

    fn publish_progress(
        &self,
        detail: String,
        progress: Option<crate::domain::language_server::InstallProgress>,
    ) -> Result<()> {
        self.check()?;
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
                server.install_progress = progress.clone();
            }
        });
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(in crate::infrastructure::language_server) struct Installed {
    pub(in crate::infrastructure::language_server) entry: PathBuf,
    pub(in crate::infrastructure::language_server) version: String,
    pub(in crate::infrastructure::language_server) source: String,
    #[serde(default)]
    pub(in crate::infrastructure::language_server) selection: Option<String>,
}

#[derive(Clone)]
pub(super) struct Binary {
    pub(super) path: PathBuf,
    pub(super) version: String,
    selection: Option<String>,
}
