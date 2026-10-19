use super::DocumentContext;
use crate::domain::document::DocumentSnapshot;
use anyhow::{Result, bail};
use ropey::Rope;
use std::{path::Path, sync::Arc};

impl DocumentContext {
    pub(crate) fn snapshots(&self) -> Vec<Arc<DocumentSnapshot>> {
        self.documents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .cloned()
            .collect()
    }

    // Recheck under the lock: the user may have edited while disk IO was running.
    pub(crate) fn refresh_from_disk_silent(
        &self,
        expected: &DocumentSnapshot,
        loaded: DocumentSnapshot,
    ) -> bool {
        {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let Some(current) = documents.get(&loaded.path) else {
                return false;
            };
            if current.saving
                || current.disk_bytes != expected.disk_bytes
                || (!current.dirty() && current.revision != expected.revision)
            {
                return false;
            }
            if current.disk_bytes == loaded.disk_bytes {
                // Clear a stale reload error when a removed/unreadable file returns.
                if current.save_error.is_none() {
                    return false;
                }
                let mut next = (**current).clone();
                next.save_error = None;
                documents.insert(next.path.clone(), Arc::new(next));
                drop(documents);
                return true;
            }
            let mut next = (**current).clone();
            if current.dirty() {
                next.save_error = Some("文件在磁盘上已修改，保留未保存的编辑，暂停自动保存".into());
                if next.save_error == current.save_error {
                    return false;
                }
                tracing::warn!(path = %loaded.path.display(), "external document change conflicts with local edits");
            } else {
                next.text = loaded.text;
                next.disk_bytes = loaded.disk_bytes;
                next.format = loaded.format;
                next.revision += 1;
                next.saved_revision = next.revision;
                next.save_error = None;
                tracing::info!(path = %loaded.path.display(), revision = next.revision, "document refreshed from disk");
            }
            documents.insert(next.path.clone(), Arc::new(next));
        }
        true
    }

    pub(crate) fn disk_refresh_failed_silent(
        &self,
        expected: &DocumentSnapshot,
        message: String,
    ) -> bool {
        let path = &expected.path;
        {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let Some(current) = documents.get(path) else {
                return false;
            };
            if current.saving
                || current.revision != expected.revision
                || current.disk_bytes != expected.disk_bytes
                || current.save_error.as_ref() == Some(&message)
            {
                return false;
            }
            let mut next = (**current).clone();
            next.save_error = Some(message);
            documents.insert(path.to_owned(), Arc::new(next));
        }
        true
    }
    pub(crate) fn release_clean(&self, path: &Path) -> bool {
        let removed = self.release_clean_silent(path);
        if removed {
            self.notify();
        }
        removed
    }
    pub(crate) fn release_clean_silent(&self, path: &Path) -> bool {
        let removed = {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if !documents
                .get(path)
                .is_some_and(|document| !document.dirty() && !document.saving)
            {
                return false;
            }
            documents.remove(path).is_some()
        };
        removed
    }
    pub fn snapshot(&self, path: &Path) -> Option<Arc<DocumentSnapshot>> {
        self.documents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(path)
            .cloned()
    }
    pub fn dirty_documents(&self) -> Vec<Arc<DocumentSnapshot>> {
        self.documents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .filter(|doc| doc.dirty())
            .cloned()
            .collect()
    }
    pub fn has_pending_save(&self) -> bool {
        self.documents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .any(|doc| doc.saving)
    }
    pub(crate) fn insert(&self, document: DocumentSnapshot) -> Arc<DocumentSnapshot> {
        let snapshot = {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            documents
                .entry(document.path.clone())
                .or_insert_with(|| Arc::new(document))
                .clone()
        };
        self.notify();
        snapshot
    }
    pub(crate) fn edit(&self, path: &Path, text: Rope, changed: bool) -> Result<()> {
        self.modify(path, |doc| {
            doc.text = text;
            if changed {
                doc.revision += 1;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub(crate) fn attach_buffer(&self, path: &Path, text: Rope, revision: u64) -> Result<()> {
        self.modify(path, |doc| {
            if doc.revision == revision {
                doc.text = text;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub(crate) fn set_autosave_paused(&self, paused: bool) {
        // Serialize with begin_save so close checks cannot miss an auto-save start.
        let _documents = self
            .documents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.autosave_paused
            .store(paused, std::sync::atomic::Ordering::Release);
    }

    pub(crate) fn autosave_paused(&self) -> bool {
        self.autosave_paused
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn begin_save(
        &self,
        path: &Path,
        automatic: bool,
        formatted: Option<(u64, Rope)>,
    ) -> Result<Option<Arc<DocumentSnapshot>>> {
        let snapshot = {
            let mut documents = self
                .documents
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let current = documents
                .get(path)
                .ok_or_else(|| anyhow::anyhow!("文件尚未打开"))?;
            if automatic
                && (self.autosave_paused()
                    || current.saving
                    || current.save_error.is_some()
                    || !current.dirty())
            {
                return Ok(None);
            }
            if current.saving {
                bail!("文件正在保存，请稍后再试");
            }
            let mut next = (**current).clone();
            next.saving = true;
            if let Some((revision, text)) = formatted {
                if current.revision == revision && current.text != text {
                    next.text = text;
                    next.revision += 1;
                    next.formatted_revision = Some(next.revision);
                    tracing::debug!(path = %path.display(), revision = next.revision, "LSP formatting applied before save");
                } else if current.revision != revision {
                    tracing::debug!(path = %path.display(), revision, current_revision = current.revision, "stale formatting discarded");
                }
            }
            next.save_error = None;
            let next = Arc::new(next);
            documents.insert(path.to_owned(), next.clone());
            next
        };
        self.notify();
        Ok(Some(snapshot))
    }
    pub(crate) fn finish_save(
        &self,
        path: &Path,
        revision: u64,
        result: Result<Arc<[u8]>>,
    ) -> Result<()> {
        self.modify(path, |doc| {
            doc.saving = false;
            match result {
                Ok(bytes) => {
                    doc.disk_bytes = bytes;
                    doc.saved_revision = revision;
                    doc.save_error = None;
                }
                Err(error) => {
                    doc.save_error = Some(format!("{error:#}"));
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}
