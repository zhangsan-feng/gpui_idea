use super::LanguageServerContext;
use crate::{
    domain::language_server::LanguageServerDataContextSnapshot,
    event_bus::refresh_gui::GuiComponent,
};
use std::sync::Arc;

impl LanguageServerContext {
    pub(super) fn replace(&self, change: impl FnOnce(&mut LanguageServerDataContextSnapshot)) {
        let mut current = self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        // Independent status/inventory publishers merge into the latest committed state.
        let mut next = (**current).clone();
        change(&mut next);
        if next == **current {
            return;
        }
        *current = Arc::new(next);
        drop(current);
        crate::data_context::DATA_CONTEXT.notify_gui_refresh(&[GuiComponent::BottomBar]);
    }
}
