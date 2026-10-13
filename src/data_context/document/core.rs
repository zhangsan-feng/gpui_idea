use super::DocumentContext;
use crate::domain::document::DocumentSnapshot;
use anyhow::{Result, anyhow};
use std::{path::Path, sync::Arc};

impl DocumentContext {
    pub(super) fn modify(
        &self,
        path: &Path,
        change: impl FnOnce(&mut DocumentSnapshot) -> Result<()>,
    ) -> Result<Arc<DocumentSnapshot>> {
        let snapshot = {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let current = documents.get(path).ok_or_else(|| anyhow!("文件尚未打开"))?;
            let mut next = (**current).clone();
            change(&mut next)?;
            let next = Arc::new(next);
            documents.insert(path.to_path_buf(), next.clone());
            next
        };
        self.notify();
        Ok(snapshot)
    }
    pub(super) fn notify(&self) {
        self.gui_refresh_notification
            .send_modify(|version| *version += 1);
    }
}
