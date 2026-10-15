use super::*;
impl Workspace {
    pub(super) fn render_terminal_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let tabs = self
            .terminal_tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let selected = index == self.active_terminal;
                h_flex()
                    .id(SharedString::from(format!("local-terminal-{index}")))
                    .h_full()
                    .px_2()
                    .gap_2()
                    .items_center()
                    .cursor_pointer()
                    .when(selected, |tab| tab.border_b_2().border_color(color(ACCENT)))
                    .hover(|style| style.bg(color(0xeef0f4)))
                    .child(tab.label.clone())
                    .child(
                        div()
                            .id(SharedString::from(format!("close-local-{index}")))
                            .w(px(18.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .hover(|style| style.bg(color(0xdfe3e8)))
                            .child(icon(IconName::X, 12., MUTED))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.close_terminal(index, cx);
                            })),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_terminal = index;
                        this.terminal_tabs[index]
                            .view
                            .update(cx, |terminal, cx| terminal.start(cx));
                        cx.notify();
                    }))
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let tab = &self.terminal_tabs[self.active_terminal];
        let snapshot = DATA_CONTEXT.terminal.snapshot(tab.id);
        v_flex()
            .size_full()
            .min_h_0()
            .border_t_1()
            .border_color(color(BORDER))
            .bg(color(EDITOR_BG))
            .child(
                h_flex()
                    .h(px(40.))
                    .flex_shrink_0()
                    .px_3()
                    .gap_2()
                    .items_center()
                    .bg(color(PANEL_BG))
                    .border_b_1()
                    .border_color(color(BORDER))
                    .child(div().font_weight(FontWeight(600.)).mr_2().child("Terminal"))
                    .children(tabs)
                    .child(
                        div()
                            .id("add-local-terminal")
                            .w(px(26.))
                            .h(px(26.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|style| style.bg(color(0xe7e9ed)))
                            .child(icon(IconName::Plus, 16., MUTED))
                            .on_click(cx.listener(|this, _, _, cx| this.add_terminal(cx))),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .id("terminal-reconnect")
                            .w(px(26.))
                            .h(px(26.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|style| style.bg(color(0xe7e9ed)))
                            .child(icon(IconName::RefreshCw, 14., MUTED))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.terminal_tabs[this.active_terminal]
                                    .view
                                    .update(cx, |terminal, cx| terminal.start(cx))
                            })),
                    )
                    .child(
                        div()
                            .id("hide-terminal")
                            .w(px(26.))
                            .h(px(26.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|style| style.bg(color(0xe7e9ed)))
                            .child(icon(IconName::Minus, 14., MUTED))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_tool(ToolWindow::Terminal, cx)
                            })),
                    ),
            )
            .child(div().flex_1().min_h_0().p_2().child(tab.view.clone()))
            .child(
                h_flex()
                    .h(px(24.))
                    .flex_shrink_0()
                    .px_3()
                    .items_center()
                    .bg(color(PANEL_BG))
                    .text_color(color(MUTED))
                    .child(if snapshot.exited {
                        "会话已结束 · 点击刷新重新打开".into()
                    } else {
                        snapshot.title.clone()
                    })
                    .child(div().flex_1())
                    .child("Ctrl+Shift+C 复制   Ctrl+Shift+V 粘贴"),
            )
            .into_any_element()
    }
}
