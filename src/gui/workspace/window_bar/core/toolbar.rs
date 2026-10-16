use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::{ClipboardItem, Context, Window};

impl WindowBar {
    pub(in crate::gui::workspace) fn toolbar_action(
        &mut self,
        action: ToolbarAction,
        window: &mut Window,
        cx: &mut Context<WindowBar>,
    ) {
        if self.state.closing || self.state.close_prompt_open {
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
            ToolbarAction::Back => self.send(crate::event_bus::gui_event::GuiEvent::Editor(
                crate::gui::workspace::editor::Event::Navigate(false),
            )),
            ToolbarAction::Forward => self.send(crate::event_bus::gui_event::GuiEvent::Editor(
                crate::gui::workspace::editor::Event::Navigate(true),
            )),
            ToolbarAction::Search => self.send(crate::event_bus::gui_event::GuiEvent::FileTree(
                crate::gui::workspace::file_tree::Event::OpenSearch,
            )),
            ToolbarAction::Tool(tool) => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::ToggleTool(tool),
                ))
            }
            ToolbarAction::RefreshGit => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::RefreshGit,
                ))
            }
            ToolbarAction::CopyBranch => cx.write_to_clipboard(ClipboardItem::new_string(
                DATA_CONTEXT.project.git().branch.clone(),
            )),
            ToolbarAction::EditRun => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::OpenRunConfiguration,
                ));
            }
            ToolbarAction::AutoRun => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::ResetRunAutomatic,
                ))
            }
            ToolbarAction::Run(action) => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::StartExecution(action),
                ))
            }
            ToolbarAction::Stop => self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                crate::gui::workspace::bottom_panel::Event::StopRun,
            )),
            ToolbarAction::CopyRunOutput => self
                .peer_bottom_panel(cx)
                .read(cx)
                .state
                .run_terminal
                .clone()
                .update(cx, |terminal, cx| terminal.copy(cx)),
            ToolbarAction::ClearRunOutput => self
                .peer_bottom_panel(cx)
                .read(cx)
                .state
                .run_terminal
                .clone()
                .update(cx, |terminal, cx| terminal.clear(cx)),
            ToolbarAction::ScrollRunToEnd => self
                .peer_bottom_panel(cx)
                .read(cx)
                .state
                .run_terminal
                .clone()
                .update(cx, |terminal, cx| terminal.scroll_to_bottom(cx)),
            ToolbarAction::Profile(profile) => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::SetRunProfile(profile),
                ))
            }
            ToolbarAction::SelectRunConfiguration(id) => {
                if let Err(error) = crate::application::APPLICATION
                    .project
                    .select_run_configuration(id)
                {
                    self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                            "{error:#}"
                        ))),
                    ));
                }
            }
            ToolbarAction::SelectRunInstance(id) => {
                if let Err(error) = crate::application::APPLICATION
                    .project
                    .select_run_instance(id)
                {
                    self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                            "{error:#}"
                        ))),
                    ));
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
                    let _ = this.update(cx, |this, _cx| {
                        if let Err(error) = result {
                            this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                                crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                    "无法重新运行：{error:#}"
                                ))),
                            ));
                        }
                        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                        ]);
                    });
                })
                .detach();
            }
        }
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
    }
}
