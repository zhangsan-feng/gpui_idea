use super::{DocumentService, OpenedDocument};
use anyhow::Result;
use ropey::Rope;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl DocumentService {
    pub(crate) async fn open_image_preview(
        &self,
        path: PathBuf,
    ) -> Result<Arc<crate::domain::image_preview::ImagePreviewSnapshot>> {
        self.open_image_core(path).await
    }
    pub(crate) async fn refresh_image_preview(
        &self,
        snapshot: Arc<crate::domain::image_preview::ImagePreviewSnapshot>,
    ) {
        self.refresh_image_core(snapshot).await;
    }
    pub(crate) fn release_image_preview(&self, path: &Path) {
        crate::data_context::DATA_CONTEXT.images.release(path);
    }
    pub fn release_clean(&self, path: &Path) -> bool {
        self.release_clean_core(path)
    }
    pub async fn open(&self, relative: PathBuf) -> Result<OpenedDocument> {
        self.open_core(relative).await
    }
    pub async fn open_external(&self, path: PathBuf) -> Result<OpenedDocument> {
        self.open_external_core(path).await
    }
    pub async fn save(&self, path: PathBuf) -> Result<()> {
        self.save_core(path).await
    }
    pub fn edit(&self, path: &Path, text: Rope) -> Result<()> {
        self.edit_core(path, text, true)
    }
    pub fn attach_editor_buffer(&self, path: &Path, text: Rope, revision: u64) -> Result<()> {
        self.attach_editor_buffer_core(path, text, revision)
    }
}
