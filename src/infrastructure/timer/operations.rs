use crate::{
    domain::document::DocumentSnapshot,
    infrastructure::{INFRASTRUCTURE, LocalChangeGuard},
};
use anyhow::Result;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::oneshot;

pub(super) async fn save_file(
    document: Arc<DocumentSnapshot>,
    completion: oneshot::Sender<Result<Arc<[u8]>>>,
) {
    tracing::debug!(path = %document.path.display(), revision = document.revision, "timer handling SaveFile event");
    let result = INFRASTRUCTURE
        .filesystem
        .save_document(document.scope_root.clone(), document)
        .await;
    let _ = completion.send(result);
}

pub(super) async fn exclude_gui_file_change(
    root: PathBuf,
    paths: Vec<PathBuf>,
    completion: oneshot::Sender<Result<LocalChangeGuard>>,
) {
    tracing::debug!(root = %root.display(), paths = paths.len(), "timer handling ExcludeGuiFileChange event");
    let result = INFRASTRUCTURE
        .filesystem
        .register_gui_file_change(root, paths)
        .await;
    let _ = completion.send(result);
}
