use super::super::ProjectService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::run::{RunConfiguration, RunConfigurationEdit, SavedRunConfiguration},
};
use anyhow::{Result, ensure};

impl ProjectService {
    pub(super) fn publish_run_configuration(&self, mut snapshot: RunConfiguration) {
        if snapshot.configurations.is_empty() {
            snapshot.selected_configuration = 1;
            snapshot.configurations.push(SavedRunConfiguration {
                id: 1,
                name: "Run".into(),
                automatic: snapshot.automatic,
                file: None,
                command: String::new(),
                selected_profile: String::new(),
                profiles: Vec::new(),
                arguments: String::new(),
                instances: 1,
                allow_parallel: false,
                error: None,
            });
        }
        if let Some(saved) = snapshot
            .configurations
            .iter_mut()
            .find(|saved| saved.id == snapshot.selected_configuration)
        {
            saved.automatic = snapshot.automatic;
            saved.file = snapshot.file.clone();
            saved.command = snapshot.command.clone();
            saved.selected_profile = snapshot.selected_profile.clone();
            saved.profiles = snapshot.profiles.clone();
            saved.error = snapshot.error.clone();
        }
        DATA_CONTEXT.project.set_run_configuration(snapshot);
    }

    pub(super) fn load_saved_run_configuration(
        snapshot: &mut RunConfiguration,
        saved: &SavedRunConfiguration,
    ) {
        snapshot.selected_configuration = saved.id;
        snapshot.automatic = saved.automatic;
        snapshot.file = saved.file.clone();
        snapshot.command = saved.command.clone();
        snapshot.selected_profile = saved.selected_profile.clone();
        snapshot.profiles = saved.profiles.clone();
        snapshot.loading = false;
        snapshot.error = saved.error.clone();
    }

    pub(in crate::application::project) fn select_run_configuration_core(
        &self,
        id: u64,
    ) -> Result<()> {
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        let saved = snapshot
            .configurations
            .iter()
            .find(|saved| saved.id == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("运行配置不存在"))?;
        Self::load_saved_run_configuration(&mut snapshot, &saved);
        snapshot.request += 1;
        self.publish_run_configuration(snapshot);
        Ok(())
    }

    pub(in crate::application::project) fn add_run_configuration_core(
        &self,
        duplicate: bool,
    ) -> Result<()> {
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        ensure!(
            !snapshot.loading && snapshot.configurations.len() < 32,
            "识别中或配置数量已达上限"
        );
        let mut saved = snapshot
            .selected()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("当前配置不存在"))?;
        saved.id = self
            .run_configuration_sequence
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let base = if duplicate {
            format!("{} 副本", saved.name)
        } else {
            "新配置".into()
        };
        saved.name = base.clone();
        let mut suffix = 2;
        while snapshot
            .configurations
            .iter()
            .any(|other| other.name == saved.name)
        {
            saved.name = format!("{base} ({suffix})");
            suffix += 1;
        }
        if !duplicate {
            saved.arguments.clear();
            saved.instances = 1;
            saved.allow_parallel = false;
        }
        Self::load_saved_run_configuration(&mut snapshot, &saved);
        snapshot.configurations.push(saved);
        snapshot.request += 1;
        self.publish_run_configuration(snapshot);
        Ok(())
    }

    pub(in crate::application::project) fn delete_run_configuration_core(
        &self,
        id: u64,
    ) -> Result<()> {
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        ensure!(snapshot.configurations.len() > 1, "至少保留一个运行配置");
        ensure!(
            snapshot.configurations.iter().any(|saved| saved.id == id),
            "运行配置不存在"
        );
        snapshot.configurations.retain(|saved| saved.id != id);
        if snapshot.selected_configuration == id {
            let next = snapshot.configurations[0].clone();
            Self::load_saved_run_configuration(&mut snapshot, &next);
        }
        snapshot.request += 1;
        self.publish_run_configuration(snapshot);
        Ok(())
    }

    pub(in crate::application::project) fn save_run_configuration_core(
        &self,
        edit: RunConfigurationEdit,
    ) -> Result<()> {
        let instances: u16 = edit
            .instances
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("实例数量必须是整数"))?;
        ensure!((1..=16).contains(&instances), "实例数量必须在 1 到 16 之间");
        let name = edit.name.trim();
        ensure!(
            !name.is_empty() && name.chars().count() <= 128,
            "配置名称不能为空且最多 128 字符"
        );
        ensure!(
            edit.command.len() <= 16_384 && !edit.command.contains('\0'),
            "运行命令过长或包含非法字符"
        );
        crate::infrastructure::INFRASTRUCTURE
            .filesystem
            .prepare_run_command("", &edit.arguments, "", None)?;
        let _guard = self
            .run_updates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut snapshot = (*DATA_CONTEXT.project.run_configuration()).clone();
        ensure!(
            !snapshot
                .configurations
                .iter()
                .any(|other| other.id != edit.id && other.name == name),
            "配置名称已存在"
        );
        let saved = snapshot
            .configurations
            .iter_mut()
            .find(|saved| saved.id == edit.id)
            .ok_or_else(|| anyhow::anyhow!("运行配置已删除"))?;
        if saved.command != edit.command {
            saved.automatic = false;
            saved.error = None;
        }
        saved.name = name.into();
        saved.command = edit.command;
        saved.arguments = edit.arguments;
        saved.instances = instances;
        saved.allow_parallel = edit.allow_parallel;
        let saved = saved.clone();
        Self::load_saved_run_configuration(&mut snapshot, &saved);
        snapshot.request += 1;
        self.publish_run_configuration(snapshot);
        Ok(())
    }
}
