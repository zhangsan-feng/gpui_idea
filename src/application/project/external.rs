use super::ProjectService;
use anyhow::Result;
use std::path::PathBuf;

impl ProjectService {
    pub(crate) async fn search_content(&self, query: String, request: u64) -> Result<()> {
        self.search_content_core(query, request).await
    }
    pub(crate) async fn file_tree_operation(
        &self,
        operation: crate::domain::file_tree::FileTreeOperation,
    ) -> Result<crate::domain::file_tree::FileTreeOutcome> {
        self.file_tree_operation_core(operation).await
    }
    pub async fn rerun_instance(&self, id: crate::domain::terminal::TerminalId) -> Result<()> {
        self.rerun_instance_core(id).await
    }
    pub fn select_run_configuration(&self, id: u64) -> Result<()> {
        self.select_run_configuration_core(id)
    }
    pub fn add_run_configuration(&self, duplicate: bool) -> Result<()> {
        self.add_run_configuration_core(duplicate)
    }
    pub fn delete_run_configuration(&self, id: u64) -> Result<()> {
        self.delete_run_configuration_core(id)
    }
    pub fn save_run_configuration(
        &self,
        edit: crate::domain::run::RunConfigurationEdit,
    ) -> Result<()> {
        self.save_run_configuration_core(edit)
    }
    pub fn select_run_instance(&self, id: crate::domain::terminal::TerminalId) -> Result<()> {
        self.select_run_instance_core(id)
    }
    pub async fn close_run_instance(&self, id: crate::domain::terminal::TerminalId) -> Result<()> {
        self.close_run_instance_core(id).await
    }
    pub async fn stop_run_instances(&self) -> Result<()> {
        self.stop_run_instances_core().await
    }
    pub async fn stop_run_configuration(&self, id: u64) -> Result<()> {
        self.stop_run_configuration_core(id).await
    }
    pub async fn initialize_run_configuration(&self) -> Result<()> {
        self.initialize_run_configuration_core().await
    }
    pub async fn refresh_run_configuration(&self, id: u64, file: Option<PathBuf>) -> Result<()> {
        self.run_configuration_core(file, Some(id)).await
    }
    pub fn select_run_profile(&self, name: &str) -> Result<()> {
        self.select_run_profile_core(name)
    }
    pub async fn execute_run(&self, action: crate::domain::run::RunAction) -> Result<()> {
        self.execute_run_core(action).await
    }
    pub async fn git_details(&self, hash: String) -> Result<()> {
        self.git_details_core(hash).await
    }
    pub async fn statistics(&self) -> Result<()> {
        self.statistics_core().await
    }
    pub async fn git_status(&self) -> Result<()> {
        self.git_status_core().await
    }
    pub async fn load_directory(&self, relative: PathBuf) -> Result<()> {
        self.load_core(relative).await
    }

    pub async fn move_entry(&self, source: PathBuf, target_directory: PathBuf) -> Result<PathBuf> {
        self.move_core(source, target_directory).await
    }
}
