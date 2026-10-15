use super::*;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    gui::workspace::ToolWindow,
};
use gpui_kit::AppContext as _;
impl Workspace {
    pub(in crate::gui::workspace) fn add_terminal(&mut self, cx: &mut Context<Self>) {
        self.terminal_sequence += 1;
        let id = crate::domain::terminal::TerminalId::Local(self.terminal_sequence);
        let view = cx.new(|cx| crate::gui::terminal::TerminalView::new(id, cx));
        view.update(cx, |terminal, cx| terminal.start(cx));
        self.terminal_tabs.push(super::super::TerminalTab {
            id,
            label: format!("Local ({})", self.terminal_sequence + 1),
            view,
        });
        self.active_terminal = self.terminal_tabs.len() - 1;
        cx.notify();
    }
    pub(in crate::gui::workspace) fn close_terminal(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = self.terminal_tabs.get(index) else {
            return;
        };
        tab.view
            .update(cx, |terminal, cx| terminal.shutdown(cx))
            .detach();
        self.terminal_tabs.remove(index);
        if index < self.active_terminal {
            self.active_terminal -= 1;
        } else {
            self.active_terminal = self
                .active_terminal
                .min(self.terminal_tabs.len().saturating_sub(1));
        }
        if self.terminal_tabs.is_empty() {
            self.terminal_sequence += 1;
            let id = crate::domain::terminal::TerminalId::Local(self.terminal_sequence);
            self.terminal_tabs.push(super::super::TerminalTab {
                id,
                label: "Local".into(),
                view: cx.new(|cx| crate::gui::terminal::TerminalView::new(id, cx)),
            });
            self.active_terminal = 0;
            self.run_panel_open = false;
            self.schedule_save(cx);
        }
        cx.notify();
    }
    pub(in crate::gui::workspace) fn toggle_tool(
        &mut self,
        tool: ToolWindow,
        cx: &mut Context<Self>,
    ) {
        self.run_panel_open = !(self.run_panel_open && self.active_tool == tool);
        self.active_tool = tool;
        if self.run_panel_open {
            if tool == ToolWindow::Terminal {
                self.terminal_tabs[self.active_terminal]
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
        tracing::info!(?tool, open = self.run_panel_open, "tool window selected");
        self.schedule_save(cx);
        cx.notify();
    }
    pub(in crate::gui::workspace) fn refresh_git(&mut self, cx: &mut Context<Self>) {
        if self.git_loading {
            return;
        }
        self.git_loading = true;
        cx.spawn(async move |this, cx| {
            let result = run_application(async { APPLICATION.project.git_status().await }).await;
            let _ = this.update(cx, |this, cx| {
                this.git_loading = false;
                if let Err(error) = result {
                    tracing::warn!(%error,"Git status failed");
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(in crate::gui::workspace) fn refresh_statistics(&mut self, cx: &mut Context<Self>) {
        if self.statistics_loading {
            return;
        }
        self.statistics_loading = true;
        cx.spawn(async move |this, cx| {
            let result = run_application(async { APPLICATION.project.statistics().await }).await;
            let _ = this.update(cx, |this, cx| {
                this.statistics_loading = false;
                if let Err(error) = result {
                    tracing::warn!(%error,"statistics refresh failed");
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(in crate::gui::workspace) fn select_git_commit(
        &mut self,
        hash: String,
        cx: &mut Context<Self>,
    ) {
        self.selected_git_commit = Some(hash.clone());
        self.git_details_loading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let requested = hash.clone();
            let result =
                run_application(async move { APPLICATION.project.git_details(hash).await }).await;
            let _ = this.update(cx, |this, cx| {
                if this.selected_git_commit.as_ref() == Some(&requested) {
                    this.git_details_loading = false;
                }
                if let Err(error) = result {
                    tracing::warn!(%error,"Git commit details failed");
                }
                cx.notify();
            });
        })
        .detach();
    }
}
