mod completion;
mod core;
mod external;
mod formatting;
mod inventory;
mod launch;
mod managed;
mod management;
mod status;

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, watch};

struct SessionSlot {
    status: status::SessionStatus,
    session: Mutex<Option<core::Session>>,
    generation: watch::Sender<u64>,
    failure: std::sync::Mutex<Option<(std::time::Instant, String)>>,
}

impl SessionSlot {
    fn new(status: status::SessionStatus) -> Self {
        Self {
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
