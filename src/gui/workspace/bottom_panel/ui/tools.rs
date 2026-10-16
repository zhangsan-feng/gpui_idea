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
        h_flex()
            .h(px(40.))
            .flex_shrink_0()
            .px_3()
            .gap_3()
            .items_center()
            .bg(color(PANEL_BG))
            .border_b_1()
            .border_color(color(BORDER))
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
                    .border_color(color(ACCENT))
                    .child(icon(symbol, 14., MUTED))
                    .child(tab)
                    .child(
                        div()
                            .id("close-tool-tab")
                            .w(px(18.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|style| style.bg(color(0xe7e9ed)))
                            .child(icon(IconName::X, 12., MUTED))
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.toggle_tool(tool, cx)),
                            ),
                    ),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("hide-tool-window")
                    .w(px(26.))
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|style| style.bg(color(0xe7e9ed)))
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
