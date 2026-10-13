use super::DocumentContext;
use crate::domain::document::DocumentSnapshot;
use anyhow::{Result, bail};
use ropey::Rope;
use std::{path::Path, sync::Arc};

impl DocumentContext {
    pub(crate) fn release_clean(&self, path: &Path) -> bool {
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
        if removed {
            self.notify();
        }
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
    pub(crate) fn begin_save(&self, path: &Path) -> Result<Arc<DocumentSnapshot>> {
        self.modify(path, |doc| {
            if doc.saving {
                bail!("文件正在保存，请稍后再试");
            }
            doc.saving = true;
            doc.save_error = None;
            Ok(())
        })
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
