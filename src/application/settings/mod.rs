mod core;
mod external;
mod language_servers;
mod system_integration;

pub(crate) struct SettingsService {
    io: tokio::sync::Mutex<()>,
    server_io: tokio::sync::RwLock<()>,
    server_snapshot_io: tokio::sync::Mutex<()>,
    system_io: tokio::sync::Mutex<()>,
}

impl SettingsService {
    pub(super) fn new() -> Self {
        Self {
            io: tokio::sync::Mutex::new(()),
            server_io: tokio::sync::RwLock::new(()),
            server_snapshot_io: tokio::sync::Mutex::new(()),
            system_io: tokio::sync::Mutex::new(()),
        }
    }
}
