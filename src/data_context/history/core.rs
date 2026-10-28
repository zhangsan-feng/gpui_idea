use super::HistoryContext;
use crate::domain::history::OpenHistory;
use crate::event_bus::refresh_gui::GuiComponent;
use std::sync::Arc;

impl HistoryContext {
    pub(super) fn replace(&self, history: OpenHistory) {
        *self
            .snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Arc::new(history);
        crate::data_context::DATA_CONTEXT
            .notify_gui_refresh(&[GuiComponent::ProjectManagerWindow, GuiComponent::WindowBar]);
    }
}
