use super::*;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::tooltip::Tooltip;

impl BottomPanel {
    pub(super) fn run_panel_header(&self, cx: &mut Context<BottomPanel>) -> Div {
        let sessions = DATA_CONTEXT.project.run_sessions();
        crate::gui::component::panel::header(40., RUN_SURFACE, RUN_BORDER).px(px(12.)).gap_0()
            .child(
                div()
                    .w(px(42.))
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .font_weight(FontWeight(600.))
                    .child("Run"),
            )
            .child(
                h_flex()
                    .id("run-instance-tabs")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_x_scroll()
                    .when(sessions.instances.is_empty(), |tabs| {
                        tabs.child(
                            div()
                                .h_full()
                                .w(px(84.))
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_size(px(12.))
                                .child(icon(IconName::CirclePlay, 14., MUTED))
                                .child("Run"),
                        )
                    })
                    .children(
                        sessions
                            .instances
                            .iter()
                            .enumerate()
                            .map(|(index, instance)| {
                                let id = instance.id;
                                let selected = sessions.selected == Some(id);
                                let snapshot = DATA_CONTEXT.terminal.snapshot(id);
                                let running = snapshot.generation > 0 && !snapshot.exited;
                                let status = if running {
                                    "正在运行".into()
                                } else {
                                    snapshot
                                        .exit_code
                                        .map(|code| {
                                            format!("Process finished with exit code {code}")
                                        })
                                        .unwrap_or_else(|| "已停止".into())
                                };
                                let hint = format!(
                                    "{}\n{}",
                                    snapshot.command.as_deref().unwrap_or(""),
                                    status
                                );
                                h_flex()
                                    .id(("run-instance-tab", index))
                                    .relative()
                                    .flex_shrink_0()
                                    .h_full()
                                    .min_w(px(100.))
                                    .px(px(10.))
                                    .gap(px(4.))
                                    .items_center()
                                    .text_size(px(12.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(crate::component::appearance::background(0xeef0f4)))
                                    .tooltip(move |window, cx| {
                                        Tooltip::new(hint.clone()).build(window, cx)
                                    })
                                    .child(icon(
                                        IconName::CirclePlay,
                                        14.,
                                        if running { 0x59a869 } else { MUTED },
                                    ))
                                    .child(instance.label.clone())
                                    .child(
                                        crate::gui::component::controls::icon_button(("close-run-instance", index), 16., 18.)
                                            .rounded_sm()
                                            .hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed)))
                                            .child(icon(IconName::X, 11., 0xa8aeb9))
                                            .on_click(cx.listener(move |this, _, _window, cx| {
                                                {
                                                    cx.stop_propagation();
                                                    this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(crate::gui::workspace::window_bar::Event::ToolbarAction(ToolbarAction::CloseRunInstance(id))));
                                                }
                                            })),
                                    )
                                    .when(selected, |tab| {
                                        tab.child(
                                            div()
                                                .absolute()
                                                .bottom_0()
                                                .left_0()
                                                .right_0()
                                                .h(px(3.))
                                                .rounded(px(1.5))
                                                .bg(crate::component::appearance::background(0x4b83ff)),
                                        )
                                    })
                                    .on_click(cx.listener(move |this, _, _window, _cx| {
                                        {
                                            this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(crate::gui::workspace::window_bar::Event::ToolbarAction(ToolbarAction::SelectRunInstance(id))))
                                        }
                                    }))
                            }),
                    ),
            )
            .child(self.run_options_menu("run-header-options", cx))
            .child(
                div()
                    .id("collapse-run-panel")
                    .w(px(26.))
                    .h(px(26.))
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed)))
                    .child(icon(IconName::Minus, 15., MUTED))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_tool(ToolWindow::Run,cx)
                    })),
            )
    }
}
