use super::{LanguageServers, SessionSlot, core, launch};
use anyhow::{Context as _, Result};
use lsp_types::{CompletionResponse, Position};
use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};
use tokio::sync::watch;

impl LanguageServers {
    pub(crate) async fn completions(
        &self,
        root: PathBuf,
        path: PathBuf,
        text: String,
        position: Position,
        language: &'static str,
        overlays: Vec<(PathBuf, String)>,
        mut cancelled: watch::Receiver<bool>,
    ) -> Result<CompletionResponse> {
        anyhow::ensure!(
            !*cancelled.borrow() && !cancelled.has_changed().is_err(),
            "补全已取消"
        );
        let family = launch::family(language)?;
        let source = path.clone();
        let root =
            tokio::task::spawn_blocking(move || launch::document_root(&root, &source, family))
                .await?;
        anyhow::ensure!(!self.stopping.load(Ordering::Acquire), "语言服务正在关闭");
        let cell = self
            .sessions
            .lock()
            .await
            .entry((root.clone(), family))
            .or_insert_with(|| std::sync::Arc::new(SessionSlot::new()))
            .clone();
        // Yield outstanding warmup/requests before synchronizing this snapshot.
        anyhow::ensure!(
            !*cancelled.borrow() && cancelled.has_changed().is_ok(),
            "补全已取消"
        );
        let mut changes = cell.generation.subscribe();
        let mut generation = 0;
        cell.generation.send_modify(|value| {
            *value = value.wrapping_add(1);
            generation = *value;
        });
        anyhow::ensure!(
            *changes.borrow_and_update() == generation,
            "补全已被较新的请求取消"
        );
        let mut session = tokio::select! {
            session = cell.session.lock() => session,
            _ = cancelled.changed() => anyhow::bail!("补全已取消"),
            _ = changes.changed() => anyhow::bail!("补全已被较新的请求取消"),
        };
        anyhow::ensure!(
            !self.stopping.load(Ordering::Acquire)
                && *changes.borrow() == generation
                && !*cancelled.borrow()
                && cancelled.has_changed().is_ok(),
            "补全已取消"
        );
        if session.as_mut().is_some_and(|session| !session.alive()) {
            if let Some(mut dead) = session.take() {
                dead.stop().await;
            }
        }
        if session.is_none() {
            *session = Some(self.start_slot(&root, family, &cell).await?);
        }
        anyhow::ensure!(
            !self.stopping.load(Ordering::Acquire)
                && *changes.borrow() == generation
                && !*cancelled.borrow()
                && cancelled.has_changed().is_ok(),
            "补全已取消"
        );
        let operation = session.as_mut().context("语言服务会话不可用")?.completions(
            path,
            text,
            position,
            language,
            overlays
                .into_iter()
                .filter(|(path, _)| family == "rust" || path.starts_with(&root))
                .collect(),
        );
        let result = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(60), operation) => Some(result),
            _ = cancelled.changed() => None,
            _ = changes.changed() => None,
        };
        let result = match result {
            Some(Ok(result)) => result,
            interrupted => {
                if let Some(session) = session.as_mut() {
                    session.cancel_current_request().await;
                }
                anyhow::bail!(if interrupted.is_none() {
                    "补全已取消"
                } else {
                    "补全等待超时"
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
