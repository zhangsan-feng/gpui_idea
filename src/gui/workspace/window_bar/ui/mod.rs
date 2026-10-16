use super::*;
use crate::domain::run::RunAction;
use crate::gui::workspace::ui::*;
use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::component::Disableable as _;

mod controls;
mod menus;
mod run_action;

impl WindowBar {
    pub(in crate::gui::workspace) fn render_toolbar(
        &self,
        window: &mut Window,
        cx: &Context<WindowBar>,
    ) -> Div {
        let running = self.run_is_active();
        h_flex()
            .w_full()
            .h(px(40.))
            .flex_shrink_0()
            .items_center()
            .gap(px(2.))
            .pl(px(8.))
            .bg(color(TOOLBAR))
            .text_color(color(0xdfe1e5))
            .child(icon(IconName::Code, 16., 0xdfe1e5))
            .child(div().w(px(10.)).flex_shrink_0())
            .child(self.main_menu(cx))
            .child(self.top_icon(
                IconName::ArrowLeft,
                "back",
                "后退",
                0xb9bdc5,
                self.peer_editor(cx).read(cx).state.navigation_index > 0,
                ToolbarAction::Back,
                cx,
            ))
            .child(self.top_icon(
                IconName::ArrowRight,
                "forward",
                "前进",
                0xb9bdc5,
                self.peer_editor(cx).read(cx).state.navigation_index + 1
                    < self.peer_editor(cx).read(cx).state.navigation.len(),
                ToolbarAction::Forward,
                cx,
            ))
            .child(self.project_menu(cx))
            .when(DATA_CONTEXT.project.git().is_repository, |toolbar| {
                toolbar.child(self.branch_menu(cx))
            })
            .child(
                div()
                    .id("window-drag-area")
                    .flex_1()
                    .h_full()
                    .min_w(px(20.))
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(self.profile_menu(cx))
            .child(self.run_action(cx))
            .child(self.top_icon(
                IconName::Hammer,
                "build",
                "构建当前配置",
                0xb9bdc5,
                self.supports_run_action(RunAction::Build, cx),
                ToolbarAction::Run(RunAction::Build),
                cx,
            ))
            .child(self.top_icon(
                IconName::Play,
                "run",
                "运行 / 重新运行当前配置",
                0x59a869,
                self.supports_run_action(RunAction::Run, cx),
                ToolbarAction::Run(RunAction::Run),
                cx,
            ))
            .child(self.top_icon(
                IconName::BugPlay,
                "debug",
                "在终端调试当前配置",
                0x59a869,
                self.supports_run_action(RunAction::Debug, cx),
                ToolbarAction::Run(RunAction::Debug),
                cx,
            ))
            .when(running, |bar| {
                bar.child(
                    self.top_icon(
                        IconName::Square,
                        "stop",
                        "停止当前配置的实例",
                        0xdb5c5c,
                        true,
                        ToolbarAction::StopRunConfiguration(
                            DATA_CONTEXT
                                .project
                                .run_configuration()
                                .selected_configuration,
                        ),
                        cx,
                    ),
                )
            })
            .child(self.more_menu(cx))
            .child(div().w(px(36.)).flex_shrink_0())
            .child(self.top_icon(
                IconName::Search,
                "search",
                "代码内容搜索",
                0xb9bdc5,
                true,
                ToolbarAction::Search,
                cx,
            ))
            .child(self.settings_menu(cx))
            .child(
                h_flex()
                    .h_full()
                    .flex_shrink_0()
                    .gap_0()
                    .child(self.window_control("minimize", IconName::Minus, window, cx))
                    .child(self.window_control("maximize", IconName::Square, window, cx))
                    .child(self.close_window_control(cx)),
            )
    }

    fn run_is_active(&self) -> bool {
        DATA_CONTEXT
            .project
            .run_sessions()
            .instances
            .iter()
            .any(|instance| {
                let snapshot = DATA_CONTEXT.terminal.snapshot(instance.id);
                instance.configuration
                    == DATA_CONTEXT
                        .project
                        .run_configuration()
                        .selected_configuration
                    && snapshot.generation > 0
                    && !snapshot.exited
            })
    }
}
