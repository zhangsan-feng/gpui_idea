use super::DataContext;

impl DataContext {
    pub(crate) fn startup(&self) -> &super::WorkspaceStartup {
        self.startup
            .get()
            .expect("工作区必须在打开 GUI 前完成初始化")
    }

    pub(crate) fn set_startup(&self, startup: super::WorkspaceStartup) -> anyhow::Result<()> {
        self.startup
            .set(startup)
            .map_err(|_| anyhow::anyhow!("工作区已经初始化"))
    }

    pub(crate) fn project_root(&self) -> &std::path::Path {
        &self.startup().project_root
    }
    pub(crate) fn subscribe_changes(&self) -> tokio::sync::watch::Receiver<u64> {
        self.changes.subscribe()
    }

    pub(crate) fn notify_gui_refresh(&self) {
        self.changes
            .send_modify(|version| *version = version.wrapping_add(1));
    }
}
