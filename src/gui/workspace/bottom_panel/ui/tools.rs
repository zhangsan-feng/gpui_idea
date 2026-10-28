use super::*;
use crate::gui::workspace::bottom_panel::BottomPanel;

impl BottomPanel {
    pub(in crate::gui::workspace::bottom_panel) fn tool_header(
        &self,
        tool: ToolWindow,
        tab: &'static str,
        symbol: IconName,
        cx: &mut Context<BottomPanel>,
    ) -> Div {
        crate::gui::component::panel::header(40., PANEL_BG, BORDER)
            .px_3()
            .gap_3()
            .child(div().font_weight(FontWeight(600.)).child(match tool {
                ToolWindow::Statistics => "Statistic",
                _ => tool.label(),
            }))
            .child(
                h_flex()
                    .h_full()
                    .px_2()
                    .gap_1()
                    .items_center()
                    .border_b_2()
                    .border_color(crate::component::appearance::border(ACCENT))
                    .child(icon(symbol, 14., MUTED))
                    .child(tab)
                    .child(
                        crate::gui::component::controls::icon_button("close-tool-tab", 18., 18.)
                            .hover(|style| {
                                style.bg(crate::component::appearance::background(0xe7e9ed))
                            })
                            .child(icon(IconName::X, 12., MUTED))
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.toggle_tool(tool, cx)),
                            ),
                    ),
            )
            .child(div().flex_1())
            .child(
                crate::gui::component::controls::icon_button("hide-tool-window", 26., 26.)
                    .cursor_pointer()
                    .hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed)))
                    .child(icon(IconName::Minus, 14., MUTED))
                    .on_click(cx.listener(move |this, _, _, cx| this.toggle_tool(tool, cx))),
            )
    }
    pub(in crate::gui::workspace) fn render_tool_panel(
        &self,
        cx: &mut Context<BottomPanel>,
    ) -> AnyElement {
        match self.state.active_tool {
            ToolWindow::Run => self.render_run_panel(cx).into_any_element(),
            ToolWindow::Statistics => self.render_statistics_panel(cx),
            ToolWindow::Git => self.render_git_panel(cx),
            ToolWindow::Terminal => self.render_terminal_panel(cx),
        }
    }
}
