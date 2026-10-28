use super::*;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::window_bar::WindowBar;

impl WindowBar {
    pub(super) fn top_icon(
        &self,
        name: IconName,
        id: &'static str,
        label: &'static str,
        tint: u32,
        enabled: bool,
        action: ToolbarAction,
        cx: &Context<WindowBar>,
    ) -> impl IntoElement {
        let config = DATA_CONTEXT.project.run_configuration();
        let hint = if enabled {
            label.to_owned()
        } else {
            match id {
                "back" | "forward" => format!("{label}：没有可用的导航记录"),
                _ if config.loading => "正在识别运行配置的工具链…".into(),
                _ => config.error.clone().unwrap_or_else(|| {
                    format!("{label}：当前配置不提供此命令，可在运行配置中编辑")
                }),
            }
        };
        crate::gui::component::controls::toolbar_button(id, cx)
            .w(px(if matches!(id, "back" | "forward") {
                40.
            } else {
                30.
            }))
            .disabled(!enabled)
            .tooltip(hint)
            .child(icon(name, 16., if enabled { tint } else { 0x60656d }))
            .on_click(cx.listener(move |this, _, window, cx| {
                if enabled {
                    this.toolbar_action(action.clone(), window, cx);
                }
            }))
    }

    pub(super) fn window_control(
        &self,
        id: &'static str,
        name: IconName,
        _window: &Window,
        _cx: &Context<WindowBar>,
    ) -> impl IntoElement {
        let area = if id == "minimize" {
            WindowControlArea::Min
        } else {
            WindowControlArea::Max
        };
        crate::gui::component::controls::icon_button(id, 38., 40.)
            .text_color(crate::component::appearance::foreground(0xd4d7dd))
            .hover(|style| style.bg(crate::component::appearance::background(TOOLBAR_HOVER)))
            .window_control_area(area)
            .child(icon(name, 12., 0xd4d7dd))
    }

    pub(super) fn close_window_control(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        crate::gui::component::controls::icon_button("close-window", 42., 40.)
            .text_color(crate::component::appearance::foreground(0xd4d7dd))
            .hover(|style| style.bg(crate::component::appearance::background(0xc42b1c)))
            .on_click(cx.listener(|this, _, window, cx| {
                tracing::info!("main window custom close requested");
                this.request_close_core(window, cx);
            }))
            .child(icon(IconName::X, 15., 0xd4d7dd))
    }
}
