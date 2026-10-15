use crate::{
    data_context::DATA_CONTEXT,
    domain::project::DirectorySnapshot,
    event_bus::{FileChanges, RefreshDataContext},
    infrastructure::INFRASTRUCTURE,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::broadcast;

pub(super) async fn refresh(changes: &FileChanges) {
    // Serialize background reloads with active mutations to avoid stale snapshot writes.
    let _guard = crate::application::APPLICATION
        .file_watch
        .lock_refresh()
        .await;
    let root = DATA_CONTEXT.project_root();
    let mut document_updates = Vec::new();
    let mut directory_updates = Vec::new();
    let affected = |path: &Path| {
        changes.rescan
            || changes
                .paths
                .iter()
                .any(|changed| path == changed || path.starts_with(changed))
    };
    for document in DATA_CONTEXT.documents.snapshots() {
        if !document.path.starts_with(root) || !affected(&document.path) || document.saving {
            continue;
        }
        let result = INFRASTRUCTURE
            .filesystem
            .read_document(document.scope_root.clone(), document.path.clone())
            .await;
        document_updates.push((document, result));
    }
    let (_, directories) = DATA_CONTEXT.project.snapshots();
    for relative in directories.keys() {
        let absolute = root.join(relative);
        if !affected(&absolute)
            && !changes
                .paths
                .iter()
                .any(|path| path.parent() == Some(absolute.as_path()))
        {
            continue;
        }
        let snapshot = match INFRASTRUCTURE
            .filesystem
            .list_directory(root.to_owned(), relative.clone())
            .await
        {
            Ok(entries) => DirectorySnapshot {
                entries: entries.into(),
                error: None,
            },
            Err(error) => DirectorySnapshot {
                entries: Vec::new().into(),
                error: Some(format!("{error:#}")),
            },
        };
        directory_updates.push((relative.clone(), snapshot));
    }
    let previous_statistics = DATA_CONTEXT.project.statistics();
    let statistics = if !previous_statistics.scanned_at.is_empty() {
        Some(
            match INFRASTRUCTURE.filesystem.statistics(root.to_owned()).await {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    tracing::warn!(%error, "watched project statistics refresh failed");
                    let mut snapshot = (*previous_statistics).clone();
                    snapshot.error = Some(format!("{error:#}"));
                    snapshot
                }
            },
        )
    } else {
        None
    };

    // Finish all IO first, then publish the batch without per-entry notifications.
    let mut updated = false;
    for (document, result) in document_updates {
        updated |= match result {
            Ok(loaded) => DATA_CONTEXT
                .documents
                .refresh_from_disk_silent(&document, loaded.document),
            Err(error) => {
                tracing::warn!(path = %document.path.display(), %error, "watched document reload failed");
                DATA_CONTEXT.documents.disk_refresh_failed_silent(
                    &document,
                    format!("磁盘文件已删除或无法重新读取：{error:#}；保留编辑器内容"),
                )
            }
        };
    }
    for (relative, snapshot) in directory_updates {
        DATA_CONTEXT
            .project
            .set_directory_silent(relative, snapshot);
        updated = true;
    }
    if let Some(snapshot) = statistics {
        DATA_CONTEXT.project.set_statistics_silent(snapshot);
        updated = true;
    }
    if updated {
        DATA_CONTEXT.notify_gui_refresh();
    }
    tracing::debug!(
        paths = changes.paths.len(),
        raw_events = changes.event_count,
        rescan = changes.rescan,
        gui_notifications = u8::from(updated),
        "filesystem changes applied"
    );
}

pub(super) async fn consume(
    root: PathBuf,
    mut events: broadcast::Receiver<Arc<RefreshDataContext>>,
) {
    loop {
        match events.recv().await {
            Ok(event) if event.root == root => {
                crate::application::APPLICATION
                    .file_watch
                    .refresh_data_context(&event.changes)
                    .await;
            }
            Ok(_) => {}
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(
                    skipped,
                    "file changes event subscriber lagged; refreshing full snapshot"
                );
                refresh(&FileChanges {
                    rescan: true,
                    event_count: skipped,
                    ..Default::default()
                })
                .await;
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}
