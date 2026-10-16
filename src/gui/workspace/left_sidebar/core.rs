use crate::gui::workspace::left_sidebar::LeftSidebar;

use gpui_kit::Context;

impl LeftSidebar {
    pub(in crate::gui::workspace) fn toggle_project_panel(
        &mut self,
        cx: &mut Context<LeftSidebar>,
    ) {
        self.state.project_panel_open = !self.state.project_panel_open;
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        tracing::debug!(open = self.state.project_panel_open, sizes = ?self.state.project_resize.read(cx).sizes(), "project panel visibility changed");
        crate::event_bus::EVENT_BUS
            .refresh_gui
            .notify(&[crate::event_bus::refresh_gui::GuiComponent::LeftSidebar]);
        self.notify_layout(cx);
        cx.notify();
    }
}
