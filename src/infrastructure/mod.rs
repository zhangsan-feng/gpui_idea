mod external;
mod history;
pub mod logging;
mod runtime;
// File-management capabilities are exposed through INFRASTRUCTURE.filesystem.
#[allow(dead_code)]
mod filesystem;

use std::sync::{LazyLock, OnceLock};

pub(crate) static INFRASTRUCTURE: LazyLock<InfrastructureContext> =
    LazyLock::new(InfrastructureContext::new);

pub(crate) struct InfrastructureContext {
    pub(crate) history: history::HistoryStore,
    pub(crate) filesystem: filesystem::FileSystem,
    runtime: OnceLock<runtime::AppRuntime>,
}

impl InfrastructureContext {
    fn new() -> Self {
        Self {
            history: history::HistoryStore::new(history::HistoryStore::default_path()),
            filesystem: filesystem::FileSystem,
            runtime: OnceLock::new(),
        }
    }
}
