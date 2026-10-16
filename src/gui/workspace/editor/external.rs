use super::*;
impl Editor {
    pub(super) fn send(&self, event: crate::event_bus::gui_event::GuiEvent) {
        crate::event_bus::EVENT_BUS
            .gui_event
            .send(self.workspace.entity_id(), event);
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

impl Editor {
    pub(in crate::gui::workspace) const WHEEL_MAX_LINES: f32 = 3.;
    pub(in crate::gui::workspace) fn restore(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(session) = DATA_CONTEXT.startup().session.clone() {
            for path in &session.open_files {
                self.select_file(ProjectFile::for_path(path), window, cx);
            }
            if let Some(active) = session
                .active_file
                .filter(|active| session.open_files.contains(active))
            {
                self.select_file(ProjectFile::for_path(&active), window, cx);
            }
        } else if let Some(path) = DATA_CONTEXT.startup().initial_file.clone() {
            self.select_file(ProjectFile(path), window, cx);
        }
    }
}
