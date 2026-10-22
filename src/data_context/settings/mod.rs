mod external;

use crate::domain::settings::GeneralSettings;

#[derive(Default)]
pub(crate) struct SettingsContext {
    value: std::sync::Mutex<GeneralSettings>,
    context_menu: std::sync::Mutex<Option<crate::domain::settings::ContextMenuStatus>>,
    servers: std::sync::Mutex<Vec<crate::domain::language_server::ManagedServer>>,
}
