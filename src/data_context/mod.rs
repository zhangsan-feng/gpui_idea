mod document;
mod external;
mod history;
mod project;
mod terminal;

use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

pub(crate) static DATA_CONTEXT: LazyLock<DataContext> = LazyLock::new(DataContext::new);

pub(crate) struct WorkspaceStartup {
    pub(crate) project_root: PathBuf,
    pub(crate) session: Option<crate::domain::history::WorkspaceSession>,
    pub(crate) notice: Option<String>,
    pub(crate) initial_file: Option<PathBuf>,
}

pub struct DataContext {
    pub terminal: terminal::TerminalContext,
    pub documents: document::DocumentContext,
    pub project: project::ProjectContext,
    pub history: history::HistoryContext,
    startup: OnceLock<WorkspaceStartup>,
    changes: tokio::sync::watch::Sender<u64>,
    language_server: std::sync::Mutex<crate::domain::language_server::LanguageServerStatus>,
}
impl DataContext {
    fn new() -> Self {
        Self {
            terminal: terminal::TerminalContext::new(),
            documents: document::DocumentContext::new(),
            project: project::ProjectContext::new(),
            history: history::HistoryContext::new(),
            startup: OnceLock::new(),
            changes: tokio::sync::watch::channel(0).0,
            language_server: Default::default(),
        }
    }
}
