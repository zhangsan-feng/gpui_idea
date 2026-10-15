use super::super::ProjectService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::run::{RunAction, RunConfiguration},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, ensure};
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

impl ProjectService {
    pub(in crate::application::project) async fn initialize_run_configuration_core(
        &self,
    ) -> Result<()> {
        if self.run_initialized.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let session = DATA_CONTEXT.startup().session.as_ref();
        let automatic = session.is_none_or(|saved| saved.run_automatic);
        let mut initial = RunConfiguration {
            automatic,
            request: if automatic { 0 } else { 1 },
            command: session
                .filter(|_| !automatic)
                .map(|saved| saved.run_command.clone())
                .unwrap_or_default(),
            selected_profile: session
                .map(|saved| saved.run_profile.clone())
                .unwrap_or_default(),
            configurations: session
                .map(|saved| saved.run_configurations.clone())
                .unwrap_or_default(),
            selected_configuration: session
                .map(|saved| saved.selected_run_configuration)
                .unwrap_or_default(),
            ..Default::default()
        };
        if let Some(saved) = initial
            .selected()
            .or_else(|| initial.configurations.first())
            .cloned()
        {
            Self::load_saved_run_configuration(&mut initial, &saved);
        }
        let automatic = initial.automatic;
        let target = initial.file.clone();
        self.run_configuration_sequence.store(
            initial
                .configurations
                .iter()
                .map(|saved| saved.id)
                .max()
                .unwrap_or(1)
                .saturating_add(1),
            Ordering::Relaxed,
        );
        self.publish_run_configuration(initial);
        if automatic {
            self.run_configuration_core(target, None).await?;
        }
        Ok(())
    }

    pub(in crate::application::project) async fn run_configuration_core(
        &self,
        file: Option<PathBuf>,
        configuration_id: Option<u64>,
    ) -> Result<()> {
        let (request, preferred) = {
            let _guard = self
                .run_updates
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let previous = DATA_CONTEXT.project.run_configuration();
            ensure!(
                configuration_id.is_none_or(|id| id == previous.selected_configuration),
                "当前运行配置已切换"
            );
            let request = previous.request + 1;
            let mut loading = (*previous).clone();
            loading.request = request;
            loading.automatic = true;
            loading.loading = true;
            loading.file = file.clone();
            loading.error = None;
            loading.command.clear();
            DATA_CONTEXT.project.set_run_configuration(loading);
            (request, previous.selected_profile.clone())
        };
        tracing::info!(?file, request, "project run configuration requested");
        let result = INFRASTRUCTURE
            .filesystem
            .run_configuration(DATA_CONTEXT.project_root().to_owned(), file.clone())
            .await;
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if DATA_CONTEXT.project.run_configuration().request != request {
            tracing::debug!(request, "superseded run configuration discarded");
            return Ok(());
        }
        match result {
            Ok(mut snapshot) => {
                snapshot.request = request;
                snapshot.automatic = true;
                let previous = DATA_CONTEXT.project.run_configuration();
                snapshot.configurations = previous.configurations.clone();
                snapshot.selected_configuration = previous.selected_configuration;
                snapshot.selected_profile = snapshot
                    .profiles
                    .iter()
                    .find(|profile| profile.name == preferred)
                    .or_else(|| snapshot.profiles.first())
                    .map(|profile| profile.name.clone())
                    .unwrap_or_default();
                snapshot.command = snapshot
                    .current_profile()
                    .map(|profile| profile.command.clone())
                    .unwrap_or_default();
                tracing::info!(?file, command = %snapshot.command, label = %snapshot.label, request,
                    profiles = ?snapshot.profiles.iter().map(|profile| profile.name.as_str()).collect::<Vec<_>>(),
                    build = snapshot.profiles.iter().any(|profile| profile.build_command.is_some()),
                    debug = snapshot.profiles.iter().any(|profile| profile.debug_command.is_some()),
                    "run configuration detected");
                self.publish_run_configuration(snapshot);
                Ok(())
            }
            Err(error) => {
                tracing::warn!(?file, error = %format!("{error:#}"), request, "run configuration detection failed");
                let previous = DATA_CONTEXT.project.run_configuration();
                self.publish_run_configuration(RunConfiguration {
                    request,
                    automatic: true,
                    file,
                    error: Some(format!("{error:#}")),
                    configurations: previous.configurations.clone(),
                    selected_configuration: previous.selected_configuration,
                    ..Default::default()
                });
                Err(error)
            }
        }
    }

    pub(in crate::application::project) fn select_run_profile_core(
        &self,
        name: &str,
    ) -> Result<()> {
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        ensure!(
            snapshot.automatic && !snapshot.loading && snapshot.error.is_none(),
            "运行配置尚不可用"
        );
        let profile = snapshot
            .profiles
            .iter()
            .find(|profile| profile.name == name)
            .ok_or_else(|| anyhow::anyhow!("运行配置不存在"))?;
        snapshot.command = profile.command.clone();
        snapshot.selected_profile = profile.name.clone();
        snapshot.request += 1;
        self.publish_run_configuration(snapshot);
        Ok(())
    }

    pub(super) fn invalidate_moved_run_target(&self, source: &Path, destination: &Path) {
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        let mut changed = false;
        let mut selected_moved = false;
        let inflight_moved = snapshot
            .loading
            .then(|| {
                snapshot
                    .file
                    .as_deref()
                    .and_then(|file| file.strip_prefix(source).ok())
                    .map(|suffix| destination.join(suffix))
            })
            .flatten();
        for saved in &mut snapshot.configurations {
            if !saved.automatic
                && !(saved.id == snapshot.selected_configuration && inflight_moved.is_some())
            {
                continue;
            }
            let target = if saved.id == snapshot.selected_configuration && inflight_moved.is_some()
            {
                inflight_moved.clone()
            } else {
                saved
                    .file
                    .as_deref()
                    .and_then(|file| file.strip_prefix(source).ok())
                    .map(|suffix| destination.join(suffix))
            };
            let Some(target) = target else {
                continue;
            };
            saved.file = Some(target);
            saved.automatic = true;
            saved.command.clear();
            saved.profiles.clear();
            saved.error = Some("运行目标已移动，请重新识别运行配置".into());
            selected_moved |= saved.id == snapshot.selected_configuration;
            changed = true;
        }
        if !changed {
            return;
        }
        if selected_moved {
            let selected = snapshot.selected().cloned().unwrap();
            Self::load_saved_run_configuration(&mut snapshot, &selected);
        } else if snapshot.loading {
            // Moving another target cannot invalidate the selected detection's request.
            DATA_CONTEXT.project.set_run_configuration(snapshot);
            return;
        }
        snapshot.request += 1;
        tracing::info!(
            ?source,
            ?destination,
            "moved run configurations invalidated"
        );
        self.publish_run_configuration(snapshot);
    }

    pub(in crate::application::project) async fn execute_run_core(
        &self,
        action: RunAction,
    ) -> Result<()> {
        self.execute_run_instances(action, None).await
    }
}
