use super::{HistoryStore, core};
use crate::domain::history::{OpenHistory, OpenKind, WorkspaceSession};
use anyhow::{Context, Result};
use std::path::PathBuf;

impl HistoryStore {
    pub async fn load(&self) -> Result<OpenHistory> {
        let path = self.path.clone();
        let lock = self.io_lock.clone();
        tokio::task::spawn_blocking(move || {
            let _guard = lock.lock().unwrap_or_else(|error| error.into_inner());
            core::read(&path)
        })
        .await
        .context("打开历史读取任务中止")?
    }

    pub async fn record(&self, kind: OpenKind, target: PathBuf) -> Result<OpenHistory> {
        let path = self.path.clone();
        let lock = self.io_lock.clone();
        tokio::task::spawn_blocking(move || {
            let _guard = lock.lock().unwrap_or_else(|error| error.into_inner());
            core::record(&path, kind, &target)
        })
        .await
        .context("打开历史写入任务中止")?
    }

    pub async fn save_session(&self, session: WorkspaceSession) -> Result<OpenHistory> {
        let path = self.path.clone();
        let lock = self.io_lock.clone();
        tokio::task::spawn_blocking(move || {
            let _guard = lock.lock().unwrap_or_else(|error| error.into_inner());
            core::save_session(&path, session)
        })
        .await
        .context("工作区状态保存任务中止")?
    }

    pub async fn validate_target(&self, kind: OpenKind, target: PathBuf) -> Result<PathBuf> {
        tokio::task::spawn_blocking(move || core::validate_target(kind, &target))
            .await
            .context("历史路径检查任务中止")?
    }
}
