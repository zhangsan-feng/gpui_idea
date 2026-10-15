use super::super::{ToolbarAction, Workspace};
use crate::data_context::DATA_CONTEXT;
use gpui_kit::{ClipboardItem, Context, Window};

impl Workspace {
    pub(in crate::gui::workspace) fn toolbar_action(
        &mut self,
        action: ToolbarAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open {
            return;
        }
        match action {
            ToolbarAction::OpenProject => self.open_project_window(
                crate::gui::project_manager::ProjectWindowKind::Open,
                window,
                cx,
            ),
            ToolbarAction::CreateProject => self.open_project_window(
                crate::gui::project_manager::ProjectWindowKind::Create,
                window,
                cx,
            ),
            ToolbarAction::ManageProjects => self.open_project_window(
                crate::gui::project_manager::ProjectWindowKind::Manage,
                window,
                cx,
            ),
            ToolbarAction::OpenRecentProject(path) => self.open_recent_project(path, cx),
            ToolbarAction::Back => self.navigate(false, window, cx),
            ToolbarAction::Forward => self.navigate(true, window, cx),
            ToolbarAction::Save => self.save_current(window, cx),
            ToolbarAction::Search => self.open_file_search(window, cx),
            ToolbarAction::ToggleProject => self.toggle_project_panel(cx),
            ToolbarAction::Tool(tool) => self.toggle_tool(tool, cx),
            ToolbarAction::RefreshProject => self.refresh_project(cx),
            ToolbarAction::RefreshGit => self.refresh_git(cx),
            ToolbarAction::CopyProjectPath => cx.write_to_clipboard(ClipboardItem::new_string(
                DATA_CONTEXT.project_root().display().to_string(),
            )),
            ToolbarAction::CopyBranch => cx.write_to_clipboard(ClipboardItem::new_string(
                DATA_CONTEXT.project.git().branch.clone(),
            )),
            ToolbarAction::EditRun => {
                self.open_run_configuration(window, cx);
            }
            ToolbarAction::AutoRun => self.reset_run_automatic(window, cx),
            ToolbarAction::Run(action) => self.start_execution(action, window, cx),
            ToolbarAction::Stop => self.stop_run(cx),
            ToolbarAction::CopyRunOutput => self
                .run_terminal
                .update(cx, |terminal, cx| terminal.copy(cx)),
            ToolbarAction::ClearRunOutput => self
                .run_terminal
                .update(cx, |terminal, cx| terminal.clear(cx)),
            ToolbarAction::ScrollRunToEnd => self
                .run_terminal
                .update(cx, |terminal, cx| terminal.scroll_to_bottom(cx)),
            ToolbarAction::Profile(profile) => self.set_run_profile(profile, window, cx),
            ToolbarAction::SelectRunConfiguration(id) => {
                if let Err(error) = crate::application::APPLICATION
                    .project
                    .select_run_configuration(id)
                {
                    self.notice = Some(format!("{error:#}"));
                }
            }
            ToolbarAction::SelectRunInstance(id) => {
                if let Err(error) = crate::application::APPLICATION
                    .project
                    .select_run_instance(id)
                {
                    self.notice = Some(format!("{error:#}"));
                }
            }
            ToolbarAction::CloseRunInstance(id) => {
                cx.spawn(async move |_, _| {
                    let _ = crate::application::lifecycle::run_application(async move {
                        crate::application::APPLICATION
                            .project
                            .close_run_instance(id)
                            .await
                    })
                    .await;
                })
                .detach();
            }
            ToolbarAction::StopAllRunInstances => {
                cx.spawn(async move |_, _| {
                    let _ = crate::application::lifecycle::run_application(async {
                        crate::application::APPLICATION
                            .project
                            .stop_run_instances()
                            .await
                    })
                    .await;
                })
                .detach();
            }
            ToolbarAction::StopRunConfiguration(id) => {
                cx.spawn(async move |_, _| {
                    let _ = crate::application::lifecycle::run_application(async move {
                        crate::application::APPLICATION
                            .project
                            .stop_run_configuration(id)
                            .await
                    })
                    .await;
                })
                .detach();
            }
            ToolbarAction::RerunRunInstance(id) => {
                cx.spawn(async move |this, cx| {
                    let result = crate::application::lifecycle::run_application(async move {
                        crate::application::APPLICATION
                            .project
                            .rerun_instance(id)
                            .await
                    })
                    .await;
                    let _ = this.update(cx, |this, cx| {
                        if let Err(error) = result {
                            this.notice = Some(format!("无法重新运行：{error:#}"));
                        }
                        cx.notify();
                    });
                })
                .detach();
            }
            ToolbarAction::CloseTabs => self.close_all_editor_tabs(cx),
            ToolbarAction::CloseWindow => self.request_close(window, cx),
        }
        cx.notify();
    }

    fn navigate(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let index = if forward {
            self.navigation_index
                .checked_add(1)
                .filter(|index| *index < self.navigation.len())
        } else {
            self.navigation_index.checked_sub(1)
        };
        let Some(index) = index else {
            return;
        };
        let file = self.navigation[index].clone();
        if self
            .moving_path
            .as_ref()
            .is_some_and(|source| file.0.starts_with(source))
        {
            return;
        }
        self.navigation_index = index;
        self.navigating = true;
        self.select_file(file, window, cx);
        self.navigating = false;
    }
}
