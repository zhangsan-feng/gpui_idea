mod document;
mod external;
mod history;
mod image_preview;
mod project;
pub(crate) mod refresh;
mod settings;
mod terminal;

use std::{
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

pub(crate) static DATA_CONTEXT: LazyLock<DataContext> = LazyLock::new(DataContext::new);

pub(crate) struct WorkspaceStartup {
    // Empty means no project. IO callers must use require_project_root().
    pub(crate) project_root: PathBuf,
    pub(crate) session: Option<crate::domain::history::WorkspaceSession>,
    pub(crate) notice: Option<String>,
    pub(crate) initial_file: Option<PathBuf>,
}

pub struct DataContext {
    pub(crate) settings: settings::SettingsContext,
    pub terminal: terminal::TerminalContext,
    pub documents: document::DocumentContext,
    pub project: project::ProjectContext,
    pub history: history::HistoryContext,
    pub(crate) images: image_preview::ImagePreviewContext,
    startup: OnceLock<WorkspaceStartup>,
    language_server: std::sync::Mutex<crate::domain::language_server::LanguageServerStatus>,
    language_servers: std::sync::Mutex<Vec<crate::domain::language_server::LanguageServerInfo>>,
}
impl DataContext {
    fn new() -> Self {
        Self {
            settings: Default::default(),
            terminal: terminal::TerminalContext::new(),
            documents: document::DocumentContext::new(),
            project: project::ProjectContext::new(),
            history: history::HistoryContext::new(),
            images: image_preview::ImagePreviewContext::default(),
            startup: OnceLock::new(),
            language_server: Default::default(),
            language_servers: Default::default(),
        }
    }
}
