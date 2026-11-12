use super::*;
use crate::gui::main_window::left_sidebar::LeftSidebar;
impl LeftSidebar {
    pub(in crate::gui) fn render_activity_bar(&self, cx: &mut Context<LeftSidebar>) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_1()
            .py_2()
            .bg(crate::component::appearance::background(ACTIVITY_BG))
            .border_r_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(self.activity_button(
                IconName::Folder,
                "project-tool",
                "项目",
                self.state.project_panel_open
                    && !(self.state.run_panel_open
                        && self.peer_bottom_panel(cx).read(cx).state.active_tool
                            == ToolWindow::Git),
                None,
                cx,
            ))
            .child(self.activity_button(
                IconName::GitBranch,
                "git-tool",
                "Git",
                self.state.run_panel_open
                    && self.peer_bottom_panel(cx).read(cx).state.active_tool == ToolWindow::Git,
                Some(ToolWindow::Git),
                cx,
            ))
            .child(div().flex_1())
            .children(
                [
                    (IconName::Play, "run-tool", ToolWindow::Run),
                    (
                        IconName::ChartNoAxesCombined,
                        "statistics-tool",
                        ToolWindow::Statistics,
                    ),
                    (IconName::Terminal, "terminal-tool", ToolWindow::Terminal),
                ]
                .into_iter()
                .map(|(icon, id, tool)| {
                    self.activity_button(
                        icon,
                        id,
                        tool.label(),
                        self.state.run_panel_open
                            && self.peer_bottom_panel(cx).read(cx).state.active_tool == tool,
                        Some(tool),
                        cx,
                    )
                    .into_any_element()
                }),
            )
    }
    fn activity_button(
        &self,
        name: IconName,
        id: &'static str,
        label: &'static str,
        selected: bool,
        tool: Option<ToolWindow>,
        cx: &mut Context<LeftSidebar>,
    ) -> impl IntoElement {
        let accented = selected && tool.is_some();
        let running = id == "run-tool"
            && DATA_CONTEXT
                .project
                .run_sessions()
                .instances
                .iter()
                .any(|instance| {
                    let snapshot = DATA_CONTEXT.terminal.snapshot(instance.id);
                    snapshot.generation > 0 && !snapshot.exited
                });
        crate::gui::component::activity_button::render(name, id, label, selected, accented, running)
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(tool) = tool {
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        this.workspace.entity_id(),
                        crate::event_bus::gui_event::GuiEvent::BottomPanel(
                            crate::gui::main_window::content_area::bottom_panel::Event::ToggleTool(
                                tool,
                            ),
                        ),
                    );
                } else if id == "project-tool" {
                    if this.state.run_panel_open
                        && this.peer_bottom_panel(cx).read(cx).state.active_tool == ToolWindow::Git
                    {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                            crate::gui::main_window::content_area::bottom_panel::Event::ToggleTool(
                                ToolWindow::Git,
                            ),
                        ));
                        if this.state.project_panel_open {
                            return;
                        }
                    }
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        this.workspace.entity_id(),
                        crate::event_bus::gui_event::GuiEvent::LeftSidebar(
                            crate::gui::main_window::left_sidebar::Event::ToggleProjectPanel,
                        ),
                    );
                }
            }))
    }
}
