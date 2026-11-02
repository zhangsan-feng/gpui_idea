use super::*;
impl FileTree {
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
    pub(super) fn peer_window_bar(
        &self,
        cx: &App,
    ) -> Entity<crate::gui::workspace::window_bar::WindowBar> {
        self.workspace
            .upgrade()
            .expect("workspace alive while component is mounted")
            .read(cx)
            .window_bar
            .clone()
    }
}

impl FileTree {
    pub(in crate::gui::workspace) fn initialize(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.load_directory(std::path::PathBuf::new(), cx);
        for path in self.state.expanded_directories.clone() {
            if !path.as_os_str().is_empty() {
                self.load_directory(path, cx);
            }
        }
    }
}

impl FileTree {}
