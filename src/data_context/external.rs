use super::DataContext;
use crate::event_bus::refresh_gui::GuiComponent;

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
    pub(crate) fn has_project(&self) -> bool {
        !self.project_root().as_os_str().is_empty()
    }
    pub(crate) fn require_project_root(&self) -> anyhow::Result<&std::path::Path> {
        anyhow::ensure!(self.has_project(), "请先打开或创建项目");
        Ok(self.project_root())
    }
    pub(crate) fn notify_gui_refresh(&self, components: &[GuiComponent]) {
        crate::event_bus::EVENT_BUS.refresh_gui.notify(components);
    }
}
