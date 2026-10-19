pub(crate) mod completion;
pub(crate) mod definition;
mod document;
mod file_watch;
mod history;
pub mod lifecycle;
mod project;
mod projects;
mod settings;
mod terminal;

use std::sync::LazyLock;

pub(crate) static APPLICATION: LazyLock<Application> = LazyLock::new(Application::new);

pub(crate) struct Application {
    pub(crate) settings: settings::SettingsService,
    pub(crate) projects: projects::ProjectsService,
    pub(crate) file_watch: file_watch::FileWatchService,
    pub(crate) terminal: terminal::TerminalService,
    pub(crate) documents: document::DocumentService,
    pub(crate) project: project::ProjectService,
    pub(crate) history: history::HistoryService,
}

impl Application {
    fn new() -> Self {
        Self {
            settings: settings::SettingsService::new(),
            projects: projects::ProjectsService,
            file_watch: file_watch::FileWatchService::new(),
            terminal: terminal::TerminalService,
            documents: document::DocumentService::new(),
            project: project::ProjectService::new(),
            history: history::HistoryService::new(),
        }
    }
}
