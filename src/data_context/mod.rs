mod document;
mod external;
mod history;
mod project;

use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};
use tokio::sync::watch;

pub(crate) static DATA_CONTEXT: LazyLock<DataContext> = LazyLock::new(DataContext::new);

pub(crate) struct WorkspaceStartup {
    pub(crate) project_root: PathBuf,
    pub(crate) session: Option<crate::domain::history::WorkspaceSession>,
    pub(crate) notice: Option<String>,
}

pub struct DataContext {
    pub documents: document::DocumentContext,
    pub project: project::ProjectContext,
    pub history: history::HistoryContext,
    gui_refresh_notification: watch::Sender<u64>,
    startup: OnceLock<WorkspaceStartup>,
}
impl DataContext {
    fn new() -> Self {
        let notification = watch::channel(0).0;
        Self {
            documents: document::DocumentContext::new(notification.clone()),
            project: project::ProjectContext::new(notification.clone()),
            history: history::HistoryContext::new(notification.clone()),
            gui_refresh_notification: notification,
            startup: OnceLock::new(),
        }
    }
}
