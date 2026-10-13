mod cache;
mod document;
mod drop;
mod lifecycle;
mod project;
mod save;
mod scroll;
mod session;
mod tabs;

use super::Workspace;
use gpui_kit::Context;

impl Workspace {
    pub(super) fn toggle_project_panel(&mut self, cx: &mut Context<Self>) {
        self.project_panel_open = !self.project_panel_open;
        self.schedule_session_save(cx);
        tracing::debug!(open = self.project_panel_open, sizes = ?self.project_resize.read(cx).sizes(), "project panel visibility changed");
        cx.notify();
    }
    pub(super) fn toggle_run_panel(&mut self, cx: &mut Context<Self>) {
        self.run_panel_open = !self.run_panel_open;
        self.schedule_session_save(cx);
        tracing::debug!(open = self.run_panel_open, sizes = ?self.run_resize.read(cx).sizes(), "run panel visibility changed");
        cx.notify();
    }
}
