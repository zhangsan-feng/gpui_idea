use super::*;
impl BottomPanel {
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

impl BottomPanel {
    pub(in crate::gui::workspace) fn initialize(&mut self, cx: &mut Context<Self>) {
        self.state.git_filter_subscription =
            Some(cx.observe(&self.state.git_filter, |_, _, cx| cx.notify()));
        self.state.run_views.insert(
            crate::domain::terminal::TerminalId::Run,
            self.state.run_terminal.clone(),
        );
        if self.peer_left_sidebar(cx).read(cx).state.run_panel_open {
            match self.state.active_tool {
                ToolWindow::Terminal => self.state.terminal_tabs[0]
                    .view
                    .update(cx, |terminal, cx| terminal.start(cx)),
                ToolWindow::Git => self.refresh_git(cx),
                ToolWindow::Statistics => self.refresh_statistics(cx),
                _ => {}
            }
        }
        self.refresh_git(cx);
    }
}
