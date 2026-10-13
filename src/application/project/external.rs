use super::ProjectService;
use anyhow::Result;
use std::path::PathBuf;

impl ProjectService {
    pub async fn load_directory(&self, relative: PathBuf) -> Result<()> {
        self.load_core(relative).await
    }

    pub async fn move_entry(&self, source: PathBuf, target_directory: PathBuf) -> Result<PathBuf> {
        self.move_core(source, target_directory).await
    }
}
