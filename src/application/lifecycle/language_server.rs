use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use std::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

pub(super) fn warm_document(snapshot: std::sync::Arc<crate::domain::document::DocumentSnapshot>) {
    if !snapshot.path.starts_with(DATA_CONTEXT.project_root())
        || !snapshot
            .path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
    {
        return;
    }
    tokio::spawn(async move {
        let path = snapshot.path.clone();
        // Rope conversion and all language-server work stay off the GUI thread.
        let text = match tokio::task::spawn_blocking(move || snapshot.text.to_string()).await {
            Ok(text) => text,
            Err(error) => {
                tracing::warn!(%error, "Preparing LSP document warmup failed");
                return;
            }
        };
        if let Err(error) = INFRASTRUCTURE
            .language_servers
            .warm_document(DATA_CONTEXT.project_root().to_owned(), path.clone(), text)
            .await
        {
            tracing::warn!(path = %path.display(), %error, "LSP document warmup failed");
        }
    });
}

pub(super) async fn start() -> anyhow::Result<()> {
    if STARTED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    let mut status = INFRASTRUCTURE.language_servers.subscribe_status();
    tokio::spawn(async move {
        loop {
            let current = status.borrow_and_update().clone();
            DATA_CONTEXT.set_language_server_status(current);
            if status.changed().await.is_err() {
                break;
            }
        }
    });
    tracing::info!(root = %DATA_CONTEXT.project_root().display(), "Prestarting workspace language server");
    INFRASTRUCTURE
        .language_servers
        .start(DATA_CONTEXT.project_root().to_owned())
        .await
}
