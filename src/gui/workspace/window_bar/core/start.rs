use super::super::*;
impl WindowBar {
    pub(in crate::gui::workspace) fn initialize(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state
            .layout_subscriptions
            .push(cx.observe_window_activation(window, |this, window, _| {
                if window.is_window_active() {
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                        crate::gui::workspace::bottom_panel::Event::RefreshGit,
                    ));
                }
            }));
        self.state.window_bounds_subscription =
            Some(cx.observe_window_bounds(window, |this, window, cx| {
                this.record_window_bounds(window, cx);
                this.update_window_view(window, cx);
            }));
        self.state
            .window_diagnostics
            .borrow_mut()
            .bounds_changed(window);
        self.state.session_ready = true;
        self.send(crate::event_bus::gui_event::GuiEvent::FileTree(
            crate::gui::workspace::file_tree::Event::Initialize,
        ));
        self.send(crate::event_bus::gui_event::GuiEvent::Editor(
            crate::gui::workspace::editor::Event::Restore,
        ));
        self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
            crate::gui::workspace::bottom_panel::Event::Initialize,
        ));
        cx.spawn(async move |_, _| {
            if let Err(error) = crate::application::lifecycle::start_language_server().await {
                tracing::warn!(%error,"workspace language server startup failed");
            }
        })
        .detach();
    }
}
