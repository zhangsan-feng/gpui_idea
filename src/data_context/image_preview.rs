use crate::domain::image_preview::{ImagePreviewDataContextSnapshot, PreviewImageFormat};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Default)]
pub(crate) struct ImagePreviewContext {
    snapshots: Mutex<HashMap<PathBuf, Arc<ImagePreviewDataContextSnapshot>>>,
    revision: AtomicU64,
}

impl ImagePreviewContext {
    pub(crate) fn snapshot(&self, path: &Path) -> Option<Arc<ImagePreviewDataContextSnapshot>> {
        self.snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(path)
            .cloned()
    }

    pub(crate) fn snapshots(&self) -> Vec<Arc<ImagePreviewDataContextSnapshot>> {
        self.snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .cloned()
            .collect()
    }

    pub(crate) fn store(
        &self,
        path: PathBuf,
        format: PreviewImageFormat,
        bytes: Arc<[u8]>,
        error: Option<String>,
    ) -> Arc<ImagePreviewDataContextSnapshot> {
        let mut snapshots = self
            .snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(current) = snapshots.get(&path) {
            if current.bytes == bytes && current.error == error {
                return current.clone();
            }
        }
        let snapshot = Arc::new(ImagePreviewDataContextSnapshot {
            path: path.clone(),
            format,
            bytes,
            error,
            revision: self.revision.fetch_add(1, Ordering::Relaxed) + 1,
        });
        snapshots.insert(path, snapshot.clone());
        snapshot
    }

    pub(crate) fn release(&self, path: &Path) {
        self.snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(path);
    }

    pub(crate) fn replace_if_current(
        &self,
        expected: &ImagePreviewDataContextSnapshot,
        bytes: Arc<[u8]>,
        error: Option<String>,
    ) {
        let mut snapshots = self
            .snapshots
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(current) = snapshots.get(&expected.path) else {
            return;
        };
        if current.revision != expected.revision
            || (current.bytes == bytes && current.error == error)
        {
            return;
        }
        snapshots.insert(
            expected.path.clone(),
            Arc::new(ImagePreviewDataContextSnapshot {
                path: expected.path.clone(),
                format: expected.format,
                bytes,
                error,
                revision: self.revision.fetch_add(1, Ordering::Relaxed) + 1,
            }),
        );
    }
}
