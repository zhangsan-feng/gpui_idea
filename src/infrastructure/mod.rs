mod external;
mod history;
pub mod logging;
mod runtime;
mod projects;
mod timer;
pub(crate) use timer::TimerRegistration;
mod terminal;
// File-management capabilities are exposed through INFRASTRUCTURE.filesystem.
#[allow(dead_code)]
mod filesystem;
pub(crate) use filesystem::FileSubscription;

use std::sync::{LazyLock, OnceLock};

pub(crate) static INFRASTRUCTURE: LazyLock<InfrastructureContext> =
    LazyLock::new(InfrastructureContext::new);

pub(crate) struct InfrastructureContext {
    pub(crate) projects: projects::ProjectStore,
    pub(crate) timer: timer::TimerPipeline,
    pub(crate) terminal: terminal::TerminalStore,
    pub(crate) history: history::HistoryStore,
    pub(crate) filesystem: filesystem::FileSystem,
    runtime: OnceLock<runtime::AppRuntime>,
}

impl InfrastructureContext {
    fn new() -> Self {
        Self {
            projects: projects::ProjectStore,
            timer: timer::TimerPipeline::new(),
            terminal: terminal::TerminalStore::new(),
            history: history::HistoryStore::new(history::HistoryStore::default_path()),
            filesystem: filesystem::FileSystem::new(),
            runtime: OnceLock::new(),
        }
    }
}
