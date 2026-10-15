use super::{EventBus, RefreshDataContext};
use std::sync::Arc;

impl EventBus {
    pub(super) fn dispatch_refresh_data_context(&self, event: RefreshDataContext) {
        tracing::debug!(root = %event.root.display(), paths = event.changes.paths.len(),
            raw_events = event.changes.event_count, rescan = event.changes.rescan,
            "event bus requesting data context refresh");
        if self.refresh_data_context.send(Arc::new(event)).is_err() {
            tracing::warn!("data context refresh requested without subscribers");
        }
    }
}
