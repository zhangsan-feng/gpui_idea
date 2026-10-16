use super::*;
use crate::gui::workspace::left_sidebar::LeftSidebar;
use gpui_kit::component::tooltip::Tooltip;
impl LeftSidebar {
    pub(in crate::gui::workspace) fn render_activity_bar(
        &self,
        cx: &mut Context<LeftSidebar>,
    ) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_1()
            .py_2()
            .bg(color(ACTIVITY_BG))
            .border_r_1()
            .border_color(color(BORDER))
            .child(self.activity_button(
                IconName::Folder,
                "project-tool",
                "项目",
                self.state.project_panel_open,
                None,
                cx,
            ))
            .child(self.activity_button(
                IconName::SlidersHorizontal,
                "structure-tool",
                "结构",
                false,
                None,
                cx,
            ))
            .child(self.activity_button(
                IconName::LayoutPanelLeft,
                "services-tool",
                "服务",
                false,
                None,
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
                    (IconName::GitBranch, "git-tool", ToolWindow::Git),
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
        div()
            .id(id)
            .relative()
            .w(px(32.))
            .h(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .bg(color(if accented {
                ACCENT
            } else if selected {
                0xdfe3e8
            } else {
                ACTIVITY_BG
            }))
            .hover(move |style| style.bg(color(if accented { 0x2866db } else { 0xe4e7eb })))
            .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
            .child(icon(
                name,
                18.,
                if accented {
                    0xffffff
                } else if selected {
                    TEXT
                } else {
                    MUTED
                },
            ))
            .when(running, |button| {
                button.child(
                    div()
                        .absolute()
                        .right(px(3.))
                        .top(px(3.))
                        .size(px(6.))
                        .rounded_full()
                        .bg(color(0x59a869)),
                )
            })
            .on_click(cx.listener(move |this, _, _, _cx| {
                if let Some(tool) = tool {
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        this.workspace.entity_id(),
                        crate::event_bus::gui_event::GuiEvent::BottomPanel(
                            crate::gui::workspace::bottom_panel::Event::ToggleTool(tool),
                        ),
                    );
                } else if id == "project-tool" {
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        this.workspace.entity_id(),
                        crate::event_bus::gui_event::GuiEvent::LeftSidebar(
                            crate::gui::workspace::left_sidebar::Event::ToggleProjectPanel,
                        ),
                    );
                }
            }))
    }
}
