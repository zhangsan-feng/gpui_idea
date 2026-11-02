use super::LanguageServers;
use anyhow::Result;
use lsp_types::{DocumentHighlight, Position};
use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};
use tokio::sync::watch;

impl LanguageServers {
    pub(crate) async fn document_highlights(
        &self,
        root: PathBuf,
        path: PathBuf,
        text: String,
        position: Position,
        language: &'static str,
        mut cancelled: watch::Receiver<bool>,
    ) -> Result<Vec<DocumentHighlight>> {
        let family = Self::document_family(&path, language)?;
        // Passive highlighting uses an existing idle session. It must never
        // start a server or interrupt completion, navigation or warmup.
        let cell = self.session_slot(root, family, false).await?;
        if self.stopping.load(Ordering::Acquire)
            || *cancelled.borrow()
            || cancelled.has_changed().is_err()
        {
            return Ok(Vec::new());
        }
        let mut session = tokio::select! {
            session = tokio::time::timeout(Duration::from_secs(2), cell.session.lock()) => {
                let Ok(session) = session else { return Ok(Vec::new()); };
                session
            }
            _ = cancelled.changed() => return Ok(Vec::new()),
        };
        if self.stopping.load(Ordering::Acquire) || *cancelled.borrow() {
            return Ok(Vec::new());
        }
        let mut changes = cell.generation.subscribe();
        let Some(active) = session.as_mut() else {
            return Ok(Vec::new());
        };
        if !active.alive() {
            return Ok(Vec::new());
        }
        let result = tokio::select! {
            result = tokio::time::timeout(Duration::from_secs(2), active.document_highlights(&path, text, position, language)) => Some(result),
            _ = cancelled.changed() => None,
            _ = changes.changed() => None,
        };
        match result {
            Some(Ok(result)) => result,
            _ => {
                active.cancel_current_request().await;
                Ok(Vec::new())
            }
        }
    }
}
