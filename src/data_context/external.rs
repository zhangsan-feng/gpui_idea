use super::DataContext;
use crate::event_bus::refresh_gui::GuiComponent;

impl DataContext {
    pub(crate) fn language_servers(
        &self,
    ) -> Vec<crate::domain::language_server::LanguageServerInfo> {
        self.language_servers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_language_servers(
        &self,
        servers: Vec<crate::domain::language_server::LanguageServerInfo>,
    ) {
        let mut current = self
            .language_servers
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *current != servers {
            *current = servers;
            drop(current);
            self.notify_gui_refresh(&[GuiComponent::BottomBar]);
        }
    }

    pub(crate) fn language_server_status(
        &self,
    ) -> crate::domain::language_server::LanguageServerStatus {
        self.language_server
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    pub(crate) fn set_language_server_status(
        &self,
        status: crate::domain::language_server::LanguageServerStatus,
    ) {
        let mut current = self
            .language_server
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *current != status {
            *current = status;
            drop(current);
            self.notify_gui_refresh(&[GuiComponent::BottomBar]);
        }
    }

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
    pub(crate) fn notify_gui_refresh(&self, components: &[GuiComponent]) {
        crate::event_bus::EVENT_BUS.refresh_gui.notify(components);
    }
}
