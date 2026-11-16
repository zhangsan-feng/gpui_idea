use super::LanguageServers;
use crate::event_bus::FileChanges;
use lsp_types::FileChangeType;
use std::path::Path;

impl LanguageServers {
    pub(crate) async fn files_changed(&self, root: &Path, changes: &FileChanges) {
        let identity = crate::domain::history::project_identity(root);
        let sessions: Vec<_> = self
            .sessions
            .lock()
            .await
            .values()
            .filter(|slot| crate::domain::history::project_identity(&slot.root) == identity)
            .cloned()
            .collect();
        if sessions.is_empty() {
            return;
        }
        if changes.rescan {
            // A dropped watcher batch has no reliable paths. Recreate lazily so
            // servers reload the complete disk state on the next interaction.
            for slot in sessions {
                let mut session = slot.session.lock().await;
                if let Some(mut session) = session.take() {
                    session.stop().await;
                }
                slot.watched_files
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .clear();
            }
            tracing::info!(root = %root.display(), "LSP sessions invalidated after filesystem rescan");
            return;
        }
        let mut events = Vec::new();
        for path in changes.paths.union(&changes.tree_paths) {
            let kind = match tokio::fs::metadata(path).await {
                Ok(_) if changes.tree_paths.contains(path) => FileChangeType::CREATED,
                Ok(_) => FileChangeType::CHANGED,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    FileChangeType::DELETED
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "LSP changed file metadata unavailable");
                    continue;
                }
            };
            events.push((path.clone(), kind));
        }
        for slot in sessions {
            slot.watched_files
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .extend(events.iter().cloned());
            // Busy sessions flush at the next document synchronization. Never
            // stall filesystem refresh behind indexing or a long LSP request.
            if let Ok(session) = slot.session.try_lock() {
                if let Some(session) = session.as_ref() {
                    if let Err(error) = session.flush_watched_files().await {
                        tracing::warn!(root = %root.display(), %error, "LSP filesystem synchronization failed");
                    }
                }
            }
        }
    }
}
