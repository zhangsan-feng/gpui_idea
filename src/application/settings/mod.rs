mod core;
mod external;
mod language_servers;

pub(crate) struct SettingsService {
    io: tokio::sync::Mutex<()>,
    server_io: tokio::sync::Mutex<()>,
}

impl SettingsService {
    pub(super) fn new() -> Self {
        Self {
            io: tokio::sync::Mutex::new(()),
            server_io: tokio::sync::Mutex::new(()),
        }
    }
}
