use crate::gui::workspace::bottom_panel::BottomPanel;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    gui::workspace::ToolWindow,
};
use gpui_kit::AppContext as _;
use gpui_kit::Context;
impl BottomPanel {
    pub(in crate::gui::workspace) fn add_terminal(&mut self, cx: &mut Context<BottomPanel>) {
        self.state.terminal_sequence += 1;
        let id = crate::domain::terminal::TerminalId::Local(self.state.terminal_sequence);
        let view = cx.new(|cx| crate::gui::terminal::TerminalView::new(id, cx));
        view.update(cx, |terminal, cx| terminal.start(cx));
        self.state
            .terminal_tabs
            .push(crate::gui::workspace::TerminalTab {
                id,
                label: format!("Local ({})", self.state.terminal_sequence + 1),
                view,
            });
        self.state.active_terminal = self.state.terminal_tabs.len() - 1;
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.notify();
    }
    pub(in crate::gui::workspace) fn close_terminal(
        &mut self,
        index: usize,
        cx: &mut Context<BottomPanel>,
    ) {
        let Some(tab) = self.state.terminal_tabs.get(index) else {
            return;
        };
        tab.view
            .update(cx, |terminal, cx| terminal.shutdown(cx))
            .detach();
        self.state.terminal_tabs.remove(index);
        if index < self.state.active_terminal {
            self.state.active_terminal -= 1;
        } else {
            self.state.active_terminal = self
                .state
                .active_terminal
                .min(self.state.terminal_tabs.len().saturating_sub(1));
        }
        if self.state.terminal_tabs.is_empty() {
            self.state.terminal_sequence += 1;
            let id = crate::domain::terminal::TerminalId::Local(self.state.terminal_sequence);
            self.state
                .terminal_tabs
                .push(crate::gui::workspace::TerminalTab {
                    id,
                    label: "Local".into(),
                    view: cx.new(|cx| crate::gui::terminal::TerminalView::new(id, cx)),
                });
            self.state.active_terminal = 0;
            self.send(crate::event_bus::gui_event::GuiEvent::LeftSidebar(
                crate::gui::workspace::left_sidebar::Event::SetRunPanel(false),
            ));
            self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                crate::gui::workspace::window_bar::Event::ScheduleSave,
            ));
        }
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.notify();
    }
    pub(in crate::gui::workspace) fn toggle_tool(
        &mut self,
        tool: ToolWindow,
        cx: &mut Context<BottomPanel>,
    ) {
        let open = !(self.peer_left_sidebar(cx).read(cx).state.run_panel_open
            && self.state.active_tool == tool);
        self.send(crate::event_bus::gui_event::GuiEvent::LeftSidebar(
            crate::gui::workspace::left_sidebar::Event::SetRunPanel(open),
        ));
        self.state.active_tool = tool;
        if open {
            if tool == ToolWindow::Terminal {
                self.state.terminal_tabs[self.state.active_terminal]
                    .view
                    .update(cx, |terminal, cx| terminal.start(cx));
            }
            if tool == ToolWindow::Git {
                self.refresh_git(cx);
            }
            if tool == ToolWindow::Statistics {
                self.refresh_statistics(cx);
            }
        }
        tracing::info!(?tool, open, "tool window selected");
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.notify();
    }
    pub(in crate::gui::workspace) fn refresh_git(&mut self, cx: &mut Context<BottomPanel>) {
        if !crate::data_context::DATA_CONTEXT.has_project() {
            return;
        }
        if self.state.git_loading {
            return;
        }
        self.state.git_loading = true;
        cx.spawn(async move |this, cx| {
            let result = run_application(async { APPLICATION.project.git_status().await }).await;
            let _ = this.update(cx, |this, cx| {
                this.state.git_loading = false;
                if let Err(error) = result {
                    tracing::warn!(%error,"Git status failed");
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                    crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                ]);
                cx.notify();
            });
        })
        .detach();
    }
    pub(in crate::gui::workspace) fn refresh_statistics(&mut self, cx: &mut Context<BottomPanel>) {
        if !crate::data_context::DATA_CONTEXT.has_project() {
            return;
        }
        if self.state.statistics_loading {
            return;
        }
        self.state.statistics_loading = true;
        cx.spawn(async move |this, cx| {
            let result = run_application(async { APPLICATION.project.statistics().await }).await;
            let _ = this.update(cx, |this, cx| {
                this.state.statistics_loading = false;
                if let Err(error) = result {
                    tracing::warn!(%error,"statistics refresh failed");
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                    crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                ]);
                cx.notify();
            });
        })
        .detach();
    }
    pub(in crate::gui::workspace) fn select_git_commit(
        &mut self,
        hash: String,
        cx: &mut Context<BottomPanel>,
    ) {
        self.state.selected_git_commit = Some(hash.clone());
        self.state.git_details_loading = true;
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        cx.notify();
        cx.spawn(async move |this, cx| {
            let requested = hash.clone();
            let result =
                run_application(async move { APPLICATION.project.git_details(hash).await }).await;
            let _ = this.update(cx, |this, cx| {
                if this.state.selected_git_commit.as_ref() == Some(&requested) {
                    this.state.git_details_loading = false;
                }
                if let Err(error) = result {
                    tracing::warn!(%error,"Git commit details failed");
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                    crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                ]);
                cx.notify();
            });
        })
        .detach();
    }
}
