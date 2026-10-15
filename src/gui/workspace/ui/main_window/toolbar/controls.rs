use super::*;
use crate::gui::workspace::ToolbarAction;
use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants as _};

impl Workspace {
    pub(super) fn toolbar_button(&self, id: &'static str, cx: &Context<Self>) -> Button {
        Button::new(id)
            // Button owns its hover/active handlers; configure their colors via the variant.
            .custom(
                ButtonCustomVariant::new(cx)
                    .foreground(color(0xdfe1e5))
                    .hover(color(TOOLBAR_HOVER))
                    .active(color(0x454a54)),
            )
            .h(px(30.))
            .px(px(6.))
            .rounded(px(4.))
            .text_size(px(13.))
            .text_color(color(0xdfe1e5))
    }

    pub(super) fn top_icon(
        &self,
        name: IconName,
        id: &'static str,
        label: &'static str,
        tint: u32,
        enabled: bool,
        action: ToolbarAction,
        cx: &Context<Self>,
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
        self.toolbar_button(id, cx)
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
        _cx: &Context<Self>,
    ) -> impl IntoElement {
        let area = if id == "minimize" {
            WindowControlArea::Min
        } else {
            WindowControlArea::Max
        };
        div()
            .id(id)
            .w(px(38.))
            .h(px(40.))
            .flex()
            .items_center()
            .justify_center()
            .text_color(color(0xd4d7dd))
            .hover(|style| style.bg(color(TOOLBAR_HOVER)))
            .window_control_area(area)
            .child(icon(name, 12., 0xd4d7dd))
    }

    pub(super) fn close_window_control(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .id("close-window")
            .w(px(42.))
            .h(px(40.))
            .flex()
            .items_center()
            .justify_center()
            .text_color(color(0xd4d7dd))
            .hover(|style| style.bg(color(0xc42b1c)))
            .on_click(cx.listener(|this, _, window, cx| {
                tracing::info!("main window custom close requested");
                this.request_close(window, cx);
            }))
            .child(icon(IconName::X, 15., 0xd4d7dd))
    }
}
