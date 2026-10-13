use super::{FileBlockEdit, FileMetadata, FileSystem, LineEdit, core};
use anyhow::Result;
use std::{path::PathBuf, sync::Arc};

impl FileSystem {
    /// All offsets refer to the original file. Expected bytes protect the target blocks.
    pub async fn modify_blocks(
        &self,
        root: PathBuf,
        relative: PathBuf,
        edits: Vec<FileBlockEdit>,
    ) -> Result<()> {
        let count = edits.len();
        tracing::debug!(path = %relative.display(), count, "filesystem block changes requested");
        core::run("modify_blocks", relative.clone(), move || {
            core::modify_blocks(&root, &relative, edits)
        })
        .await
    }

    pub async fn metadata(&self, root: PathBuf, relative: PathBuf) -> Result<FileMetadata> {
        core::run("metadata", relative.clone(), move || {
            core::metadata(&root, &relative)
        })
        .await
    }
    pub async fn read_file(
        &self,
        root: PathBuf,
        relative: PathBuf,
        max_bytes: usize,
    ) -> Result<Vec<u8>> {
        core::run("read", relative.clone(), move || {
            core::read_file(&root, &relative, max_bytes)
        })
        .await
    }
    pub async fn create_file(
        &self,
        root: PathBuf,
        relative: PathBuf,
        contents: Vec<u8>,
    ) -> Result<()> {
        core::run("create_file", relative.clone(), move || {
            core::create_file(&root, &relative, &contents)
        })
        .await
    }
    pub async fn write_file(
        &self,
        root: PathBuf,
        relative: PathBuf,
        contents: Vec<u8>,
        expected: Vec<u8>,
    ) -> Result<()> {
        core::run("write_file", relative.clone(), move || {
            core::write_file(&root, &relative, &contents, &expected)
        })
        .await
    }

    /// Applies a batch against bytes previously read by the caller. All positions
    /// refer to the original file. Returns the bytes written as the next baseline.
    pub async fn modify_lines(
        &self,
        root: PathBuf,
        relative: PathBuf,
        expected: Vec<u8>,
        edits: Vec<LineEdit>,
    ) -> Result<Vec<u8>> {
        let count = edits.len();
        tracing::debug!(path = %relative.display(), count, "filesystem line changes requested");
        core::run("modify_lines", relative.clone(), move || {
            core::modify_lines(&root, &relative, &expected, edits)
        })
        .await
    }
    pub async fn create_directory(&self, root: PathBuf, relative: PathBuf) -> Result<()> {
        core::run("create_directory", relative.clone(), move || {
            core::create_directory(&root, &relative)
        })
        .await
    }
    pub async fn rename(&self, root: PathBuf, source: PathBuf, destination: PathBuf) -> Result<()> {
        tracing::debug!(source = %source.display(), destination = %destination.display(), "filesystem rename requested");
        core::run("rename", source.clone(), move || {
            core::rename(&root, &source, &destination)
        })
        .await
    }
    pub async fn remove_file(&self, root: PathBuf, relative: PathBuf) -> Result<()> {
        core::run("remove_file", relative.clone(), move || {
            core::remove_file(&root, &relative)
        })
        .await
    }
    pub async fn remove_directory(&self, root: PathBuf, relative: PathBuf) -> Result<()> {
        core::run("remove_directory", relative.clone(), move || {
            core::remove_directory(&root, &relative)
        })
        .await
    }
    pub async fn set_readonly(
        &self,
        root: PathBuf,
        relative: PathBuf,
        readonly: bool,
    ) -> Result<()> {
        core::run("set_readonly", relative.clone(), move || {
            core::set_readonly(&root, &relative, readonly)
        })
        .await
    }

    pub async fn list_directory(
        &self,
        root: PathBuf,
        relative: PathBuf,
    ) -> Result<Vec<crate::domain::project::ProjectEntry>> {
        super::directory::read(root, relative).await
    }
    pub async fn read_document(
        &self,
        root: PathBuf,
        path: PathBuf,
    ) -> Result<super::document::LoadedFile> {
        super::document::read(root, path).await
    }
    pub async fn save_document(
        &self,
        root: PathBuf,
        document: Arc<crate::domain::document::DocumentSnapshot>,
    ) -> Result<Arc<[u8]>> {
        super::document::save(root, document).await
    }
}
