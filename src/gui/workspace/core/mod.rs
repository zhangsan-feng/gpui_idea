mod cache;
mod document;
mod definition;
mod document_sync;
mod drop;
mod file_tree_menu;
mod inline_file_edit;
mod lifecycle;
mod project;
mod projects;
mod run;
mod save;
mod scroll;
mod search;
mod session;
mod tabs;
mod toolbar;
mod tools;

use super::Workspace;
use gpui_kit::Context;

impl Workspace {
    pub(super) fn toggle_project_panel(&mut self, cx: &mut Context<Self>) {
        self.project_panel_open = !self.project_panel_open;
        self.schedule_save(cx);
        tracing::debug!(open = self.project_panel_open, sizes = ?self.project_resize.read(cx).sizes(), "project panel visibility changed");
        cx.notify();
    }
    pub(super) fn toggle_run_panel(&mut self, cx: &mut Context<Self>) {
        self.toggle_tool(super::ToolWindow::Run, cx);
    }
}
