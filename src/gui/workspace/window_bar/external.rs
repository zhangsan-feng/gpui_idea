use super::*;
impl WindowBar {
    pub(super) fn send(&self, event: crate::event_bus::gui_event::GuiEvent) {
        crate::event_bus::EVENT_BUS
            .gui_event
            .send(self.workspace.entity_id(), event);
    }

    pub(super) fn peer_editor(&self, cx: &App) -> Entity<crate::gui::workspace::editor::Editor> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .editor
            .clone()
    }
    pub(super) fn peer_file_tree(
        &self,
        cx: &App,
    ) -> Entity<crate::gui::workspace::file_tree::FileTree> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .file_tree
            .clone()
    }

    pub(super) fn peer_bottom_panel(
        &self,
        cx: &App,
    ) -> Entity<crate::gui::workspace::bottom_panel::BottomPanel> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .bottom_panel
            .clone()
    }
    pub(super) fn peer_left_sidebar(
        &self,
        cx: &App,
    ) -> Entity<crate::gui::workspace::left_sidebar::LeftSidebar> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .left_sidebar
            .clone()
    }
}

impl WindowBar {
    pub(super) fn current_run_profile(&self) -> Option<crate::domain::run::RunProfile> {
        DATA_CONTEXT
            .project
            .run_configuration()
            .current_profile()
            .cloned()
    }
    pub(super) fn supports_run_action(
        &self,
        action: crate::domain::run::RunAction,
        _cx: &Context<Self>,
    ) -> bool {
        !DATA_CONTEXT.project.run_sessions().starting
            && DATA_CONTEXT
                .project
                .run_configuration()
                .command_for(action)
                .is_some()
    }
}
