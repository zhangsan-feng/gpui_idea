mod core;
mod external;
mod launch;

use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, watch};

struct SessionSlot {
    session: Mutex<Option<core::Session>>,
    generation: watch::Sender<u64>,
}

pub(crate) struct LanguageServers {
    sessions: Mutex<HashMap<(PathBuf, &'static str), Arc<SessionSlot>>>,
    status: watch::Sender<crate::domain::language_server::LanguageServerStatus>,
    stopping: std::sync::atomic::AtomicBool,
}
