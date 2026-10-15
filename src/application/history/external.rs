use super::HistoryService;
use crate::domain::history::{OpenKind, WorkspaceSession};
use anyhow::Result;
use std::path::PathBuf;

impl HistoryService {
    pub(crate) async fn forget_project(&self, root: PathBuf) -> Result<()> {
        self.forget_project_core(root).await
    }
    pub(crate) async fn restore_project(&self, root: PathBuf) -> Result<Option<WorkspaceSession>> {
        self.restore_project_core(root).await
    }
    pub async fn load(&self) -> Result<()> {
        self.load_core().await
    }
    pub async fn record_opened(&self, kind: OpenKind, path: PathBuf) -> Result<()> {
        self.record_core(kind, path).await
    }
    pub async fn save_session(&self, session: WorkspaceSession) -> Result<()> {
        self.save_core(session).await
    }
    pub async fn restore_session(&self) -> Result<Option<WorkspaceSession>> {
        self.restore_core().await
    }
}
