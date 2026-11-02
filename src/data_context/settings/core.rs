use super::SettingsContext;
use crate::domain::settings::SettingsDataContextSnapshot;
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::Arc;

impl SettingsContext {
    pub(super) fn replace(
        &self,
        change: impl FnOnce(&mut SettingsDataContextSnapshot),
        components: &[GuiComponent],
    ) {
        let mut current = self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // Merge only the changed field into the latest snapshot under the same lock.
        let mut next = (**current).clone();
        change(&mut next);
        if next == **current {
            return;
        }
        *current = Arc::new(next);
        drop(current);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(components);
    }
}
