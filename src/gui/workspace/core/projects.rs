use super::super::Workspace;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    gui::project_manager::{self, ProjectWindowKind},
};
use gpui_kit::Context;
use std::path::PathBuf;

impl Workspace {
    pub(in crate::gui::workspace) fn open_project_window(
        &mut self,
        mode: ProjectWindowKind,
        parent: &gpui_kit::Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(window) = self.project_windows.get(&mode) {
            if window
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        match project_manager::open(mode, parent, cx) {
            Ok(window) => {
                self.project_windows.insert(mode, window);
            }
            Err(error) => self.notice = Some(format!("无法打开项目窗口：{error:#}")),
        }
    }
    pub(in crate::gui::workspace) fn open_recent_project(
        &mut self,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        if self.closing {
            return;
        }
        if path
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .eq_ignore_ascii_case(
                crate::data_context::DATA_CONTEXT
                    .project_root()
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\"),
            )
        {
            self.notice = Some("项目已在当前工作区打开".into());
            return;
        }
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.projects.open(path).await }).await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.notice = Some(format!("无法打开项目：{error:#}"));
                }
                cx.notify();
            });
        })
        .detach();
    }
}
