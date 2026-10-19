use super::{LanguageServers, core, launch};
use anyhow::{Context as _, Result};
use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};
use tokio::time::{Instant, timeout_at};

impl LanguageServers {
    pub(crate) async fn format_document(
        &self,
        root: PathBuf,
        path: PathBuf,
        text: String,
        language: &'static str,
    ) -> Result<Vec<lsp_types::TextEdit>> {
        let deadline = Instant::now() + Duration::from_secs(15);
        let family = launch::family(language)?;
        let source = path.clone();
        let root = timeout_at(
            deadline,
            tokio::task::spawn_blocking(move || launch::document_root(&root, &source, family)),
        )
        .await
        .context("格式化等待超时")??;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let sessions = timeout_at(deadline, self.sessions.lock())
            .await
            .context("格式化等待超时")?;
        // File opening owns server startup/installation. Saving must never
        // initiate an installation or wait indefinitely for a warming session.
        let cell = sessions
            .get(&(root, family))
            .cloned()
            .context("语言服务尚未准备好")?;
        drop(sessions);
        let mut changes = cell.generation.subscribe();
        let mut session = tokio::select! {
            session = timeout_at(deadline, cell.session.lock()) => session.context("格式化等待超时")?,
            _ = changes.changed() => anyhow::bail!("格式化让位于新的语言服务请求"),
        };
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let active = session.as_mut().context("语言服务尚未准备好")?;
        anyhow::ensure!(active.alive(), "语言服务尚未准备好");
        let result = tokio::select! {
            result = timeout_at(deadline, active.format_document(&path, text, language)) => Some(result),
            _ = changes.changed() => None,
        };
        let result = match result {
            Some(Ok(result)) => result,
            interrupted => {
                active.cancel_current_request().await;
                anyhow::bail!(if interrupted.is_none() {
                    "格式化已取消"
                } else {
                    "格式化等待超时"
                });
            }
        };
        if result
            .as_ref()
            .is_err_and(|error| error.is::<core::SessionFault>())
        {
            if let Some(mut failed) = session.take() {
                failed.stop().await;
            }
        }
        result
    }
}
