mod completion;
mod core;
mod external;
mod inventory;
mod launch;
mod managed;

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, watch};

struct SessionSlot {
    session: Mutex<Option<core::Session>>,
    generation: watch::Sender<u64>,
    failure: std::sync::Mutex<Option<(std::time::Instant, String)>>,
}

impl SessionSlot {
    fn new() -> Self {
        Self {
            session: Mutex::new(None),
            generation: watch::channel(0).0,
            failure: std::sync::Mutex::new(None),
        }
    }
}

pub(crate) struct LanguageServers {
    sessions: Mutex<HashMap<(PathBuf, &'static str), Arc<SessionSlot>>>,
    status: watch::Sender<crate::domain::language_server::LanguageServerStatus>,
    inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    stopping: Arc<std::sync::atomic::AtomicBool>,
    updates_started: std::sync::atomic::AtomicBool,
}
