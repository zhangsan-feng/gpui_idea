use super::*;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    menu::DropdownMenu,
    tooltip::Tooltip,
};

impl BottomPanel {
    pub(super) fn run_toolbar(&self, cx: &mut Context<BottomPanel>) -> Div {
        let sessions = DATA_CONTEXT.project.run_sessions();
        let configuration = DATA_CONTEXT.project.run_configuration();
        let rerun = sessions
            .selected
            .and_then(|id| sessions.instances.iter().find(|instance| instance.id == id));
        let enabled = if let Some(instance) = rerun {
            !sessions.starting
                && configuration.configurations.iter().any(|saved| {
                    saved.id == instance.configuration
                        && saved.error.is_none()
                        && !saved.command.trim().is_empty()
                })
        } else {
            self.supports_run_action(crate::domain::run::RunAction::Run, cx)
        };
        let snapshot = DATA_CONTEXT.terminal.snapshot(
            DATA_CONTEXT
                .project
                .run_sessions()
                .selected
                .unwrap_or(crate::domain::terminal::TerminalId::Run),
        );
        let running = snapshot.generation > 0 && !snapshot.exited;
        h_flex()
            .h(px(36.))
            .flex_shrink_0()
            .px(px(8.))
            .gap_0()
            .items_center()
            .bg(crate::component::appearance::background(RUN_SURFACE))
            .border_b_1()
            .border_color(crate::component::appearance::border(RUN_BORDER))
            .child(
                self.run_button(
                    if snapshot.generation > 0 {
                        IconName::RefreshCw
                    } else {
                        IconName::Play
                    },
                    "start-run",
                    "运行 / 重新运行",
                    0x59a869,
                    enabled,
                    rerun
                        .map(|instance| ToolbarAction::RerunRunInstance(instance.id))
                        .unwrap_or(ToolbarAction::Run(crate::domain::run::RunAction::Run)),
                    cx,
                ),
            )
            .child(self.run_button(
                IconName::Square,
                "stop-run",
                "停止程序",
                0xf04468,
                running,
                ToolbarAction::Stop,
                cx,
            ))
            .child(crate::gui::component::panel::separator(18., 6., RUN_BORDER))
            .child(self.run_button(
                IconName::ArrowDownToLine,
                "scroll-run-to-end",
                "滚动到底部",
                MUTED,
                snapshot.generation > 0,
                ToolbarAction::ScrollRunToEnd,
                cx,
            ))
            .child(self.run_button(
                IconName::Trash,
                "clear-run-output",
                "清空控制台",
                MUTED,
                snapshot.generation > 0,
                ToolbarAction::ClearRunOutput,
                cx,
            ))
    }

    fn run_button(
        &self,
        symbol: IconName,
        id: &'static str,
        label: &'static str,
        tint: u32,
        enabled: bool,
        action: ToolbarAction,
        cx: &mut Context<BottomPanel>,
    ) -> AnyElement {
        let rerun = id == "start-run" && symbol == IconName::RefreshCw;
        crate::gui::component::controls::icon_button(id, 24., 24.)
            .rounded(px(3.))
            .when(enabled, |button| {
                button
                    .cursor_pointer()
                    .hover(|s| s.bg(crate::component::appearance::background(0xe8eaed)))
            })
            .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
            .child(
                div()
                    .relative()
                    .size(px(16.))
                    .child(icon(
                        symbol,
                        14.,
                        if !enabled {
                            0xb8bdc6
                        } else if rerun {
                            MUTED
                        } else {
                            tint
                        },
                    ))
                    .when(rerun, |symbol| {
                        symbol.child(
                            div()
                                .absolute()
                                .right_0()
                                .bottom_0()
                                .size(px(8.))
                                .bg(crate::component::appearance::background(RUN_SURFACE))
                                .child(icon(
                                    IconName::Play,
                                    8.,
                                    if enabled { tint } else { 0xb8bdc6 },
                                )),
                        )
                    }),
            )
            .on_click(cx.listener(move |this, _, _window, _cx| {
                if enabled {
                    this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                        crate::gui::workspace::window_bar::Event::ToolbarAction(action.clone()),
                    ));
                }
            }))
            .into_any_element()
    }

    pub(super) fn run_options_menu(
        &self,
        id: &'static str,
        cx: &Context<BottomPanel>,
    ) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let snapshot = DATA_CONTEXT.terminal.snapshot(
            DATA_CONTEXT
                .project
                .run_sessions()
                .selected
                .unwrap_or(crate::domain::terminal::TerminalId::Run),
        );
        let has_output = snapshot.generation > 0;
        let running = has_output && !snapshot.exited;
        let all_running = DATA_CONTEXT
            .project
            .run_sessions()
            .instances
            .iter()
            .any(|instance| {
                let snapshot = DATA_CONTEXT.terminal.snapshot(instance.id);
                snapshot.generation > 0 && !snapshot.exited
            });
        let automatic = DATA_CONTEXT.project.run_configuration().automatic;
        Button::new(id)
            .ghost()
            .w(px(24.))
            .h(px(24.))
            .px_0()
            .rounded(px(3.))
            .tooltip("Run 选项")
            .child(icon(IconName::EllipsisVertical, 14., MUTED))
            .dropdown_menu(move |menu, _, _| {
                menu.item(Self::toolbar_item(
                    "复制控制台文本",
                    ToolbarAction::CopyRunOutput,
                    has_output,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "清空控制台",
                    ToolbarAction::ClearRunOutput,
                    has_output,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "滚动到底部",
                    ToolbarAction::ScrollRunToEnd,
                    has_output,
                    false,
                    &view,
                ))
                .separator()
                .item(Self::toolbar_item(
                    "编辑运行配置…",
                    ToolbarAction::EditRun,
                    true,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "自动识别当前文件 / 项目",
                    ToolbarAction::AutoRun,
                    true,
                    automatic,
                    &view,
                ))
                .separator()
                .item(Self::toolbar_item(
                    "停止程序",
                    ToolbarAction::Stop,
                    running,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "停止所有实例",
                    ToolbarAction::StopAllRunInstances,
                    all_running,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "隐藏 Run 面板",
                    ToolbarAction::Tool(ToolWindow::Run),
                    true,
                    false,
                    &view,
                ))
            })
    }
}
