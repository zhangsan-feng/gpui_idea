use super::{EventBus, FileChanges, RefreshDataContext};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::broadcast;

impl EventBus {
    pub(crate) fn refresh_data_context(&self, root: PathBuf, changes: FileChanges) {
        self.dispatch_refresh_data_context(RefreshDataContext { root, changes });
    }

    pub(crate) fn subscribe_refresh_data_context(
        &self,
    ) -> broadcast::Receiver<Arc<RefreshDataContext>> {
        self.refresh_data_context.subscribe()
    }
}
