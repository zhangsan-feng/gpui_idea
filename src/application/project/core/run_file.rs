use super::super::ProjectService;
use crate::{
    application::APPLICATION,
    data_context::DATA_CONTEXT,
    domain::run::{RunAction, SavedRunConfiguration},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, ensure};
use std::{path::PathBuf, sync::atomic::Ordering};

impl ProjectService {
    pub(in crate::application::project) async fn run_file_core(&self, file: PathBuf) -> Result<()> {
        ensure!(
            crate::domain::run::supports_file(&file),
            "此文件类型尚不支持运行"
        );
        APPLICATION.documents.save_dirty().await?;
        let detected = INFRASTRUCTURE
            .filesystem
            .file_run_configuration(DATA_CONTEXT.project_root().to_owned(), file.clone())
            .await?;
        ensure!(
            detected
                .profiles
                .iter()
                .any(|profile| !profile.command.is_empty()),
            "未找到可运行的入口，请选择 main 文件、二进制目标或脚本文件"
        );
        let id = {
            let _guard = self
                .run_updates
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
            ensure!(!snapshot.loading, "正在识别运行配置，请稍后重试");
            let existing = snapshot
                .configurations
                .iter()
                .find(|saved| saved.automatic && saved.file.as_ref() == Some(&file))
                .cloned();
            ensure!(
                existing.is_some() || snapshot.configurations.len() < 32,
                "运行配置数量已达上限"
            );
            let mut saved = existing.unwrap_or_else(|| SavedRunConfiguration {
                id: self
                    .run_configuration_sequence
                    .fetch_add(1, Ordering::Relaxed),
                name: file.to_string_lossy().into_owned(),
                automatic: true,
                file: Some(file.clone()),
                command: String::new(),
                selected_profile: String::new(),
                profiles: vec![],
                arguments: String::new(),
                instances: 1,
                allow_parallel: false,
                error: None,
            });
            saved.profiles = detected.profiles;
            let profile = saved
                .profiles
                .iter()
                .find(|profile| profile.name == saved.selected_profile)
                .filter(|profile| !profile.command.is_empty())
                .or_else(|| {
                    saved
                        .profiles
                        .iter()
                        .find(|profile| !profile.command.is_empty())
                })
                .unwrap();
            saved.selected_profile = profile.name.clone();
            saved.command = profile.command.clone();
            saved.error = None;
            let id = saved.id;
            snapshot.configurations.retain(|other| other.id != id);
            snapshot.configurations.push(saved.clone());
            Self::load_saved_run_configuration(&mut snapshot, &saved);
            snapshot.label = detected.label;
            snapshot.request += 1;
            self.publish_run_configuration(snapshot);
            id
        };
        tracing::info!(file = %file.display(), configuration = id, "file tree run requested");
        self.execute_run_instances(RunAction::Run, None, Some(id))
            .await
    }
}
