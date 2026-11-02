mod core;
mod external;

use crate::domain::settings::SettingsDataContextSnapshot;

#[derive(Default)]
pub(crate) struct SettingsContext {
    snapshot: std::sync::Mutex<std::sync::Arc<SettingsDataContextSnapshot>>,
}
