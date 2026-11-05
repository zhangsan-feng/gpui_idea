use super::super::ProjectService;
use crate::{application::APPLICATION, data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Notify;

static STARTED: AtomicBool = AtomicBool::new(false);
static REFRESH: Notify = Notify::const_new();

impl ProjectService {
    pub(crate) fn schedule_file_run_refresh(&self) {
        if !STARTED.swap(true, Ordering::AcqRel) {
            tokio::spawn(async {
                loop {
                    REFRESH.notified().await;
                    refresh().await;
                }
            });
        }
        REFRESH.notify_one();
    }
}

async fn refresh() {
    let root = DATA_CONTEXT.project_root().to_owned();
    // Restored tabs and search results can open files without expanding their parents.
    // Populate the shared tree cache so both editor and tree menus use the same capability.
    for document in DATA_CONTEXT.documents.snapshots() {
        if document.scope_root != root {
            continue;
        }
        let Ok(relative) = document.path.strip_prefix(&root) else {
            continue;
        };
        if !INFRASTRUCTURE.language_servers.supports_file_run(relative) {
            continue;
        }
        let Some(directory) = relative.parent() else {
            continue;
        };
        let _guard = APPLICATION.file_watch.lock_refresh().await;
        if DATA_CONTEXT.project_root() != root {
            return;
        }
        let (_, directories) = DATA_CONTEXT.project.file_tree.snapshots();
        if directories.get(directory).is_some_and(|snapshot| {
            snapshot.error.is_none()
                && snapshot
                    .entries
                    .iter()
                    .any(|entry| entry.relative_path == relative)
        }) {
            continue;
        }
        match INFRASTRUCTURE
            .filesystem
            .list_directory(root.clone(), directory.to_owned())
            .await
        {
            Ok(entries) if DATA_CONTEXT.project_root() == root => {
                tracing::info!(file = %relative.display(), directory = %directory.display(), "opened document run capability directory loaded");
                DATA_CONTEXT.project.file_tree.set_directory(
                    directory.to_owned(),
                    crate::domain::project::DirectoryDataContextSnapshot {
                        entries: entries.into(),
                        error: None,
                    },
                );
            }
            Ok(_) => return,
            Err(error) => {
                tracing::warn!(file = %relative.display(), %error, "opened document run capability directory load failed");
            }
        }
    }
    let (_, directories) = DATA_CONTEXT.project.file_tree.snapshots();
    for (directory, original) in directories {
        let mut entries = original.entries.to_vec();
        let mut changed = false;
        for entry in &mut entries {
            if entry.is_directory
                || entry.is_link
                || !INFRASTRUCTURE
                    .language_servers
                    .has_server_file_run_support(&entry.relative_path)
            {
                continue;
            }
            match INFRASTRUCTURE
                .language_servers
                .server_file_run_configuration(root.clone(), entry.relative_path.clone(), false)
                .await
            {
                Ok(Some(target)) => {
                    let runnable = target.is_some();
                    changed |= entry.runnable != runnable;
                    entry.runnable = runnable;
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::debug!(file = %entry.relative_path.display(), %error, "LSP file run capability refresh deferred")
                }
            }
        }
        if !changed {
            continue;
        }
        let _guard = APPLICATION.file_watch.lock_refresh().await;
        let (_, current) = DATA_CONTEXT.project.file_tree.snapshots();
        // A filesystem refresh may have replaced/deleted these entries while LSP was busy.
        if DATA_CONTEXT.project_root() != root
            || !current
                .get(&directory)
                .is_some_and(|snapshot| Arc::ptr_eq(snapshot, &original))
        {
            continue;
        }
        DATA_CONTEXT.project.file_tree.set_directory(
            directory,
            crate::domain::project::DirectoryDataContextSnapshot {
                entries: entries.into(),
                error: original.error.clone(),
            },
        );
    }
}
