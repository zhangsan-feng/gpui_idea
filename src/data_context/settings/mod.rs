mod core;
mod external;

use crate::domain::settings::SettingsDataContextSnapshot;

#[derive(Default)]
pub(crate) struct SettingsContext {
    snapshot: std::sync::Mutex<std::sync::Arc<SettingsDataContextSnapshot>>,
    // Zero means there is no wheel adjustment waiting to be saved.
    pending_editor_font_size: std::sync::atomic::AtomicU16,
}
