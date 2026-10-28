use super::super::ProjectService;
use crate::{
    application::APPLICATION,
    data_context::DATA_CONTEXT,
    domain::{
        run::{RunAction, RunInstance},
        terminal::TerminalId,
    },
    infrastructure::INFRASTRUCTURE,
};
use anyhow::{Result, ensure};
use std::sync::atomic::Ordering;

impl ProjectService {
    pub(in crate::application::project) async fn stop_run_configuration_core(
        &self,
        id: u64,
    ) -> Result<()> {
        let _execution = self.run_executions.lock().await;
        for instance in DATA_CONTEXT
            .project
            .run_sessions()
            .instances
            .iter()
            .filter(|instance| instance.configuration == id)
        {
            APPLICATION.terminal.stop(instance.id).await?;
        }
        Ok(())
    }
    pub(super) async fn execute_run_instances(
        &self,
        action: RunAction,
        target: Option<TerminalId>,
        configuration_id: Option<u64>,
    ) -> Result<()> {
        let _execution = self.run_executions.lock().await;
        let mut configuration = (*DATA_CONTEXT.project.run_configuration()).clone();
        if let Some(id) = configuration_id {
            let saved = configuration.configurations.iter().find(|saved| saved.id == id)
                .cloned().ok_or_else(|| anyhow::anyhow!("运行配置已删除"))?;
            Self::load_saved_run_configuration(&mut configuration, &saved);
        }
        if let Some(target) = target {
            let sessions = DATA_CONTEXT.project.run_sessions();
            let instance = sessions
                .instances
                .iter()
                .find(|instance| instance.id == target)
                .ok_or_else(|| anyhow::anyhow!("运行实例已关闭"))?;
            let saved = configuration
                .configurations
                .iter()
                .find(|saved| saved.id == instance.configuration)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("实例的运行配置已删除"))?;
            Self::load_saved_run_configuration(&mut configuration, &saved);
        }
        let saved = configuration
            .selected()
            .ok_or_else(|| anyhow::anyhow!("运行配置不存在"))?;
        let base = configuration
            .command_for(action)
            .ok_or_else(|| anyhow::anyhow!("当前配置不提供此命令"))?;
        let profile = configuration.current_profile();
        let command = if action != RunAction::Build {
            INFRASTRUCTURE.filesystem.prepare_run_command(
                base,
                &saved.arguments,
                profile
                    .map(|profile| {
                        if action == RunAction::Debug {
                            profile.debug_argument_prefix.as_str()
                        } else {
                            profile.argument_prefix.as_str()
                        }
                    })
                    .unwrap_or(""),
                profile.and_then(|profile| {
                    if action == RunAction::Debug {
                        profile.debug_argument_template.as_deref()
                    } else {
                        profile.argument_template.as_deref()
                    }
                }),
            )?
        } else {
            base.to_owned()
        };
        let count = if action == RunAction::Run && target.is_none() {
            saved.instances
        } else {
            1
        };
        ensure!((1..=16).contains(&count), "实例数量必须在 1 到 16 之间");
        let current = DATA_CONTEXT.project.run_sessions();
        let replace = if let Some(target) = target {
            vec![target]
        } else {
            current
                .instances
                .iter()
                .filter(|instance| instance.configuration == saved.id && !saved.allow_parallel)
                .map(|instance| instance.id)
                .collect::<Vec<_>>()
        };
        let running = current
            .instances
            .iter()
            .filter(|instance| !replace.contains(&instance.id))
            .filter(|instance| {
                let snapshot = DATA_CONTEXT.terminal.snapshot(instance.id);
                snapshot.generation > 0 && !snapshot.exited
            })
            .count();
        ensure!(
            running + usize::from(count) <= 32,
            "运行中的实例最多 32 个，请先停止部分实例"
        );
        {
            let _guard = self
                .run_sessions_updates
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let mut sessions = (*DATA_CONTEXT.project.run_sessions()).clone();
            sessions.starting = true;
            DATA_CONTEXT.project.set_run_sessions(sessions);
        }
        let mut created = Vec::new();
        let result = async {
            for id in replace {
                APPLICATION.terminal.stop(id).await?;
            }
            for index in 0..count {
                let id = target.unwrap_or_else(|| {
                    TerminalId::RunInstance(
                        self.run_instance_sequence.fetch_add(1, Ordering::Relaxed),
                    )
                });
                let sequence = match id {
                    TerminalId::RunInstance(sequence) => sequence,
                    _ => unreachable!(),
                };
                APPLICATION
                    .terminal
                    .start(id, Some(command.clone()))
                    .await?;
                created.push(id);
                let mut retired = Vec::new();
                {
                    let _guard = self
                        .run_sessions_updates
                        .lock()
                        .unwrap_or_else(|error| error.into_inner());
                    let mut sessions = (*DATA_CONTEXT.project.run_sessions()).clone();
                    while sessions.instances.len() >= 48 {
                        let Some(old) = sessions.instances.iter().position(|instance| {
                            DATA_CONTEXT.terminal.snapshot(instance.id).exited
                        }) else {
                            break;
                        };
                        retired.push(sessions.instances.remove(old).id);
                    }
                    if !sessions.instances.iter().any(|instance| instance.id == id) {
                        sessions.instances.push(RunInstance {
                            id,
                            configuration: saved.id,
                            label: format!("{} ({sequence})", saved.name),
                        });
                    }
                    if index == 0 {
                        sessions.selected = Some(id);
                    }
                    DATA_CONTEXT.project.set_run_sessions(sessions);
                }
                for retired in retired {
                    APPLICATION.terminal.release(retired).await?;
                }
                tracing::info!(
                    ?id,
                    configuration = saved.id,
                    ?action,
                    index,
                    count,
                    "run instance started"
                );
            }
            Ok(())
        }
        .await;
        if result.is_err() {
            for id in created {
                let _ = APPLICATION.terminal.stop(id).await;
            }
        }
        let _guard = self
            .run_sessions_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut sessions = (*DATA_CONTEXT.project.run_sessions()).clone();
        sessions.starting = false;
        DATA_CONTEXT.project.set_run_sessions(sessions);
        result
    }

    pub(in crate::application::project) fn select_run_instance_core(
        &self,
        id: TerminalId,
    ) -> Result<()> {
        let _guard = self
            .run_sessions_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut sessions = (*DATA_CONTEXT.project.run_sessions()).clone();
        ensure!(
            sessions.instances.iter().any(|instance| instance.id == id),
            "运行实例不存在"
        );
        sessions.selected = Some(id);
        DATA_CONTEXT.project.set_run_sessions(sessions);
        Ok(())
    }
    pub(in crate::application::project) async fn rerun_instance_core(
        &self,
        id: TerminalId,
    ) -> Result<()> {
        self.execute_run_instances(RunAction::Run, Some(id), None).await
    }

    pub(in crate::application::project) async fn close_run_instance_core(
        &self,
        id: TerminalId,
    ) -> Result<()> {
        let _execution = self.run_executions.lock().await;
        APPLICATION.terminal.release(id).await?;
        let _guard = self
            .run_sessions_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut sessions = (*DATA_CONTEXT.project.run_sessions()).clone();
        sessions.instances.retain(|instance| instance.id != id);
        if sessions.selected == Some(id) {
            sessions.selected = sessions.instances.last().map(|instance| instance.id);
        }
        DATA_CONTEXT.project.set_run_sessions(sessions);
        Ok(())
    }

    pub(in crate::application::project) async fn stop_run_instances_core(&self) -> Result<()> {
        let _execution = self.run_executions.lock().await;
        for instance in &DATA_CONTEXT.project.run_sessions().instances {
            APPLICATION.terminal.stop(instance.id).await?;
        }
        Ok(())
    }
}
