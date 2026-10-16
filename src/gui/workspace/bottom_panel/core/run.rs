use crate::domain::run::RunAction;
use crate::gui::workspace::ToolWindow;
use crate::gui::workspace::bottom_panel::BottomPanel;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::{AppContext as _, Context, Window};

impl BottomPanel {
    pub(in crate::gui::workspace) fn open_run_configuration(
        &mut self,
        window: &mut Window,
        cx: &mut Context<BottomPanel>,
    ) {
        if let Some(handle) = self.state.run_configuration_window {
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        let parent = self.workspace.clone();
        let parent_window = window.window_handle();
        let result = crate::gui::external::open_tool_window(
            "运行 / 调试配置",
            gpui_kit::size(gpui_kit::px(840.), gpui_kit::px(520.)),
            window,
            cx,
            move |window, cx| {
                cx.new(|cx| {
                    crate::gui::workspace::run_editor::RunConfigurationEditor::new(
                        parent,
                        parent_window,
                        window,
                        cx,
                    )
                })
            },
        );
        match result {
            Ok((window, _)) => self.state.run_configuration_window = Some(window),
            Err(error) => self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                    "无法打开运行配置窗口：{error:#}"
                ))),
            )),
        }
    }

    pub(in crate::gui::workspace) fn sync_run_configuration(
        &mut self,
        window: &mut Window,
        _cx: &mut Context<BottomPanel>,
    ) {
        let snapshot = DATA_CONTEXT.project.run_configuration();
        if self.state.run_config_revision == snapshot.request {
            return;
        }
        self.state.run_config_revision = snapshot.request;
        let _ = window;
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
    }

    pub(in crate::gui::workspace) fn reset_run_automatic(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<BottomPanel>,
    ) {
        let file = (!self.peer_editor(cx).read(cx).state.open_files.is_empty())
            .then(|| self.peer_editor(cx).read(cx).state.selected_file.0.clone());
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

    pub(in crate::gui::workspace) fn supports_run_action(
        &self,
        action: RunAction,
        _cx: &Context<BottomPanel>,
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
        cx: &mut Context<BottomPanel>,
    ) {
        if let Err(error) = APPLICATION.project.select_run_profile(&profile) {
            self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                    "无法选择配置：{error:#}"
                ))),
            ));
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            ]);
            cx.notify();
        }
    }

    pub(in crate::gui::workspace) fn start_execution(
        &mut self,
        action: RunAction,
        _window: &mut Window,
        cx: &mut Context<BottomPanel>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || !self.supports_run_action(action, cx)
        {
            return;
        }
        self.state.active_tool = ToolWindow::Run;
        self.send(crate::event_bus::gui_event::GuiEvent::LeftSidebar(
            crate::gui::workspace::left_sidebar::Event::SetRunPanel(true),
        ));
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.execute_run(action).await }).await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                            "无法启动：{error:#}"
                        ))),
                    ));
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                    crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                    crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
                cx.notify();
            });
        })
        .detach();
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
        cx.notify();
    }

    pub(in crate::gui::workspace) fn stop_run(&mut self, cx: &mut Context<BottomPanel>) {
        self.state
            .run_terminal
            .update(cx, |terminal, cx| terminal.stop(cx));
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
        cx.notify();
    }

    pub(in crate::gui::workspace) fn sync_run_views(&mut self, cx: &mut Context<BottomPanel>) {
        let sessions = DATA_CONTEXT.project.run_sessions();
        self.state.run_views.retain(|id, _| {
            *id == crate::domain::terminal::TerminalId::Run
                || sessions.instances.iter().any(|instance| instance.id == *id)
        });
        for instance in &sessions.instances {
            self.state.run_views.entry(instance.id).or_insert_with(|| {
                cx.new(|cx| crate::gui::terminal::TerminalView::new(instance.id, cx))
            });
        }
        let selected = sessions
            .selected
            .unwrap_or(crate::domain::terminal::TerminalId::Run);
        if let Some(view) = self.state.run_views.get(&selected) {
            self.state.run_terminal = view.clone();
        }
    }
}
