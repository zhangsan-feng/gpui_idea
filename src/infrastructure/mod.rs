pub(crate) mod assets;
mod external;
mod git;
mod history;
mod language_server;
pub mod logging;
pub(crate) mod paths;
mod projects;
mod runtime;
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
    pub(crate) git: git::Git,
    pub(crate) projects: projects::ProjectStore,
    pub(crate) timer: timer::TimerPipeline,
    pub(crate) terminal: terminal::TerminalStore,
    pub(crate) history: history::HistoryStore,
    pub(crate) filesystem: filesystem::FileSystem,
    pub(crate) language_servers: language_server::LanguageServers,
    runtime: OnceLock<runtime::AppRuntime>,
}

impl InfrastructureContext {
    fn new() -> Self {
        Self {
            git: git::Git,
            projects: projects::ProjectStore,
            timer: timer::TimerPipeline::new(),
            terminal: terminal::TerminalStore::new(),
            history: history::HistoryStore::new(history::HistoryStore::default_path()),
            filesystem: filesystem::FileSystem::new(),
            language_servers: language_server::LanguageServers::new(),
            runtime: OnceLock::new(),
        }
    }
}
