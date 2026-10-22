use super::*;

impl SettingsContext {
    pub(crate) fn context_menu(&self) -> Option<crate::domain::settings::ContextMenuStatus> {
        self.context_menu
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_context_menu(
        &self,
        status: Option<crate::domain::settings::ContextMenuStatus>,
    ) {
        *self
            .context_menu
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = status;
    }

    pub(crate) fn servers(&self) -> Vec<crate::domain::language_server::ManagedServer> {
        self.servers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_servers(&self, servers: Vec<crate::domain::language_server::ManagedServer>) {
        *self
            .servers
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = servers;
    }

    pub(crate) fn get(&self) -> GeneralSettings {
        self.value
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set(&self, value: GeneralSettings) {
        *self.value.lock().unwrap_or_else(|error| error.into_inner()) = value;
    }
}
