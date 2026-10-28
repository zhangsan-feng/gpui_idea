use super::*;
use crate::gui::workspace::bottom_panel::BottomPanel;
impl BottomPanel {
    pub(super) fn render_terminal_panel(&self, cx: &mut Context<BottomPanel>) -> AnyElement {
        let tabs = self
            .state
            .terminal_tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let selected = index == self.state.active_terminal;
                h_flex()
                    .id(SharedString::from(format!("local-terminal-{index}")))
                    .h_full()
                    .px_2()
                    .gap_2()
                    .items_center()
                    .cursor_pointer()
                    .when(selected, |tab| {
                        tab.border_b_2()
                            .border_color(crate::component::appearance::border(ACCENT))
                    })
                    .hover(|style| style.bg(crate::component::appearance::background(0xeef0f4)))
                    .child(tab.label.clone())
                    .child(
                        crate::gui::component::controls::icon_button(
                            SharedString::from(format!("close-local-{index}")),
                            18.,
                            18.,
                        )
                        .rounded_sm()
                        .hover(|style| style.bg(crate::component::appearance::background(0xdfe3e8)))
                        .child(icon(IconName::X, 12., MUTED))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            cx.stop_propagation();
                            this.close_terminal(index, cx);
                        })),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.state.active_terminal = index;
                        this.state.terminal_tabs[index]
                            .view
                            .update(cx, |terminal, cx| terminal.start(cx));
                        crate::event_bus::EVENT_BUS
                            .refresh_gui
                            .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomPanel]);
                    }))
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let tab = &self.state.terminal_tabs[self.state.active_terminal];
        let snapshot = DATA_CONTEXT.terminal.snapshot(tab.id);
        v_flex()
            .size_full()
            .min_h_0()
            .border_t_1()
            .border_color(crate::component::appearance::border(BORDER))
            .bg(crate::component::appearance::background(EDITOR_BG))
            .child(
                crate::gui::component::panel::header(40., PANEL_BG, BORDER)
                    .px_3()
                    .gap_2()
                    .child(div().font_weight(FontWeight(600.)).mr_2().child("Terminal"))
                    .children(tabs)
                    .child(
                        crate::gui::component::controls::icon_button(
                            "add-local-terminal",
                            26.,
                            26.,
                        )
                        .cursor_pointer()
                        .hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed)))
                        .child(icon(IconName::Plus, 16., MUTED))
                        .on_click(cx.listener(|this, _, _, cx| this.add_terminal(cx))),
                    )
                    .child(div().flex_1())
                    .child(
                        crate::gui::component::controls::icon_button(
                            "terminal-reconnect",
                            26.,
                            26.,
                        )
                        .cursor_pointer()
                        .hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed)))
                        .child(icon(IconName::RefreshCw, 14., MUTED))
                        .on_click(cx.listener(|this, _, _, cx| {
                            {
                                this.state.terminal_tabs[this.state.active_terminal]
                                    .view
                                    .update(cx, |terminal, cx| terminal.start(cx))
                            }
                        })),
                    )
                    .child(
                        crate::gui::component::controls::icon_button("hide-terminal", 26., 26.)
                            .cursor_pointer()
                            .hover(|style| {
                                style.bg(crate::component::appearance::background(0xe7e9ed))
                            })
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
                    .bg(crate::component::appearance::background(PANEL_BG))
                    .text_color(crate::component::appearance::foreground(MUTED))
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
