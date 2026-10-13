use super::{DocumentService, OpenedDocument};
use anyhow::Result;
use ropey::Rope;
use std::path::{Path, PathBuf};

impl DocumentService {
    pub fn release_clean(&self, path: &Path) -> bool {
        self.release_clean_core(path)
    }
    pub async fn open(&self, relative: PathBuf) -> Result<OpenedDocument> {
        let opened = self.open_core(relative).await?;
        self.record_opened(&opened).await;
        Ok(opened)
    }
    pub async fn open_external(&self, path: PathBuf) -> Result<OpenedDocument> {
        let opened = self.open_external_core(path).await?;
        self.record_opened(&opened).await;
        Ok(opened)
    }
    pub async fn save(&self, path: PathBuf) -> Result<()> {
        self.save_core(path).await
    }
    pub fn edit(&self, path: &Path, text: Rope) -> Result<()> {
        self.edit_core(path, text, true)
    }
    pub fn attach_editor_buffer(&self, path: &Path, text: Rope) -> Result<()> {
        self.edit_core(path, text, false)
    }
}
