use super::FileRefreshBatch;
use crate::data_context::DataContext;

impl DataContext {
    pub(crate) fn apply_file_refresh(&self, batch: FileRefreshBatch) {
        self.apply_file_refresh_core(batch);
    }
}
