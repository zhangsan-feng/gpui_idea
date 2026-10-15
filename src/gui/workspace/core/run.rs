use super::super::{ToolWindow, Workspace};
use crate::domain::run::{RunAction, RunProfile};
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::{AppContext as _, Context, Window};

impl Workspace {
    pub(in crate::gui::workspace) fn open_run_configuration(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(handle) = self.run_configuration_window {
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        let parent = cx.entity().downgrade();
        let parent_window = window.window_handle();
        let result = crate::gui::external::open_tool_window(
            "运行 / 调试配置",
            gpui_kit::size(gpui_kit::px(840.), gpui_kit::px(520.)),
            window,
            cx,
            move |window, cx| {
                cx.new(|cx| {
                    super::super::run_editor::RunConfigurationEditor::new(
                        parent,
                        parent_window,
                        window,
                        cx,
                    )
                })
            },
        );
        match result {
            Ok((window, _)) => self.run_configuration_window = Some(window),
            Err(error) => self.notice = Some(format!("无法打开运行配置窗口：{error:#}")),
        }
    }

    pub(in crate::gui::workspace) fn sync_run_configuration(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snapshot = DATA_CONTEXT.project.run_configuration();
        if self.run_config_revision == snapshot.request {
            return;
        }
        self.run_config_revision = snapshot.request;
        let _ = window;
        self.schedule_save(cx);
    }

    pub(in crate::gui::workspace) fn reset_run_automatic(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let file = (!self.open_files.is_empty()).then(|| self.selected_file.0.clone());
        let id = DATA_CONTEXT
            .project
            .run_configuration()
            .selected_configuration;
        cx.spawn(async move |_, _| {
            let _ = run_application(async move {
                APPLICATION
                    .project
                    .refresh_run_configuration(id, file)
                    .await
            })
            .await;
        })
        .detach();
    }

    pub(in crate::gui::workspace) fn current_run_profile(&self) -> Option<RunProfile> {
        DATA_CONTEXT
            .project
            .run_configuration()
            .current_profile()
            .cloned()
    }

    pub(in crate::gui::workspace) fn supports_run_action(
        &self,
        action: RunAction,
        _cx: &Context<Self>,
    ) -> bool {
        !DATA_CONTEXT.project.run_sessions().starting
            && DATA_CONTEXT
                .project
                .run_configuration()
                .command_for(action)
                .is_some()
    }

    pub(in crate::gui::workspace) fn set_run_profile(
        &mut self,
        profile: String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = APPLICATION.project.select_run_profile(&profile) {
            self.notice = Some(format!("无法选择配置：{error:#}"));
            cx.notify();
        }
    }

    pub(in crate::gui::workspace) fn start_execution(
        &mut self,
        action: RunAction,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open || !self.supports_run_action(action, cx) {
            return;
        }
        self.active_tool = ToolWindow::Run;
        self.run_panel_open = true;
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.execute_run(action).await }).await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.notice = Some(format!("无法启动：{error:#}"));
                }
                cx.notify();
            });
        })
        .detach();
        self.schedule_save(cx);
        cx.notify();
    }

    pub(in crate::gui::workspace) fn stop_run(&mut self, cx: &mut Context<Self>) {
        self.run_terminal
            .update(cx, |terminal, cx| terminal.stop(cx));
        cx.notify();
    }

    pub(in crate::gui::workspace) fn sync_run_views(&mut self, cx: &mut Context<Self>) {
        let sessions = DATA_CONTEXT.project.run_sessions();
        self.run_views.retain(|id, _| {
            *id == crate::domain::terminal::TerminalId::Run
                || sessions.instances.iter().any(|instance| instance.id == *id)
        });
        for instance in &sessions.instances {
            self.run_views.entry(instance.id).or_insert_with(|| {
                cx.new(|cx| crate::gui::terminal::TerminalView::new(instance.id, cx))
            });
        }
        let selected = sessions
            .selected
            .unwrap_or(crate::domain::terminal::TerminalId::Run);
        if let Some(view) = self.run_views.get(&selected) {
            self.run_terminal = view.clone();
        }
    }
}
