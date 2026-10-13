mod document;
mod history;
pub mod lifecycle;
mod project;

use std::sync::LazyLock;

pub(crate) static APPLICATION: LazyLock<Application> = LazyLock::new(Application::new);

pub(crate) struct Application {
    pub(crate) documents: document::DocumentService,
    pub(crate) project: project::ProjectService,
    pub(crate) history: history::HistoryService,
}

impl Application {
    fn new() -> Self {
        Self {
            documents: document::DocumentService::new(),
            project: project::ProjectService::new(),
            history: history::HistoryService::new(),
        }
    }
}
