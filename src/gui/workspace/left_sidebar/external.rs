use super::*;
impl LeftSidebar {
    pub(super) fn send(&self, event: crate::event_bus::gui_event::GuiEvent) {
        crate::event_bus::EVENT_BUS
            .gui_event
            .send(self.workspace.entity_id(), event);
    }
    pub(super) fn notify_layout(&self, cx: &mut App) {
        gpui_kit::App::notify(cx, self.workspace.entity_id());
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
}
