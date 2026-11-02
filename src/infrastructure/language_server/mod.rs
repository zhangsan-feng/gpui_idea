mod completion;
mod core;
mod external;
mod formatting;
mod highlights;
mod inventory;
mod languages;
mod launch;
mod managed;
mod management;
mod manager;
mod packages;
mod runnables;
mod status;
pub(in crate::infrastructure) use packages::is_code_package_directory;
pub(in crate::infrastructure) use runnables::local_file_run_configuration;

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, watch};

struct SessionSlot {
    root: PathBuf,
    status: status::SessionStatus,
    session: Mutex<Option<core::Session>>,
    generation: watch::Sender<u64>,
    failure: std::sync::Mutex<Option<(std::time::Instant, String)>>,
}

impl SessionSlot {
    fn new(root: PathBuf, status: status::SessionStatus) -> Self {
        Self {
            root,
            status,
            session: Mutex::new(None),
            generation: watch::channel(0).0,
            failure: std::sync::Mutex::new(None),
        }
    }
}

pub(crate) struct LanguageServers {
    sessions: Mutex<HashMap<(PathBuf, &'static str), Arc<SessionSlot>>>,
    status: status::StatusRegistry,
    inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    stopping: Arc<std::sync::atomic::AtomicBool>,
    updates_started: std::sync::atomic::AtomicBool,
}
