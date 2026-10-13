use super::*;
use crate::data_context::DATA_CONTEXT;

impl Workspace {
    pub(super) fn render_toolbar(&self, window: &mut Window, cx: &Context<Self>) -> Div {
        h_flex()
            .w_full()
            .h(px(40.))
            .flex_shrink_0()
            .items_center()
            .gap(px(4.))
            .pl(px(7.))
            .bg(color(TOOLBAR))
            .text_color(color(0xe6e8ec))
            .child(self.top_icon(IconName::Menu, "menu", cx))
            .child(self.top_icon(IconName::ArrowLeft, "back", cx))
            .child(self.top_icon(IconName::ArrowRight, "forward", cx))
            .child(
                h_flex()
                    .h(px(30.))
                    .px_2()
                    .gap_2()
                    .items_center()
                    .rounded_sm()
                    .hover(|style| style.bg(color(TOOLBAR_HOVER)))
                    .child(
                        div()
                            .w(px(22.))
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_sm()
                            .bg(color(0x159cab))
                            .text_size(px(11.))
                            .font_weight(FontWeight(700.))
                            .text_color(color(0xffffff))
                            .child("GI"),
                    )
                    .child(
                        div().text_sm().font_weight(FontWeight(600.)).child(
                            DATA_CONTEXT
                                .project_root()
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned(),
                        ),
                    )
                    .child(icon(IconName::ChevronDown, 13., 0xaab0ba)),
            )
            .child(self.top_icon(IconName::GitBranch, "branch", cx))
            .child(div().text_xs().text_color(color(0xaeb4be)).child("master"))
            .child(icon(IconName::ChevronDown, 12., 0x9299a4))
            .child(
                div()
                    .id("window-drag-area")
                    .flex_1()
                    .h_full()
                    .min_w(px(40.))
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                h_flex()
                    .h(px(30.))
                    .px_2()
                    .gap_1()
                    .items_center()
                    .rounded_sm()
                    .hover(|style| style.bg(color(TOOLBAR_HOVER)))
                    .child(div().text_sm().child("dev"))
                    .child(icon(IconName::ChevronDown, 12., 0xaeb4be)),
            )
            .child(self.run_action(cx))
            .child(self.top_icon(IconName::Hammer, "build", cx))
            .child(self.top_icon(IconName::BugPlay, "debug", cx))
            .child(self.top_icon(IconName::EllipsisVertical, "more", cx))
            .child(div().w(px(8.)))
            .child(self.top_icon(IconName::Search, "search", cx))
            .child(self.top_icon(IconName::Settings, "settings", cx))
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

    fn top_icon(&self, name: IconName, id: &'static str, _cx: &Context<Self>) -> impl IntoElement {
        div()
            .id(id)
            .w(px(30.))
            .h(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .text_color(color(0xc9cdd4))
            .hover(|style| style.bg(color(TOOLBAR_HOVER)))
            .child(icon(name, 16., 0xc9cdd4))
    }

    fn run_action(&self, cx: &Context<Self>) -> impl IntoElement {
        h_flex()
            .id("run-panel-action")
            .h(px(30.))
            .px_2()
            .gap_1()
            .items_center()
            .rounded_sm()
            .text_color(color(0xe6e8ec))
            .hover(|style| style.bg(color(TOOLBAR_HOVER)))
            .child(icon(IconName::CirclePlay, 16., 0xb7bdc7))
            .child(div().text_sm().child("Run"))
            .child(icon(IconName::ChevronDown, 12., 0xaeb4be))
            .on_click(cx.listener(|this, _, _, cx| this.toggle_run_panel(cx)))
    }

    fn window_control(
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

    fn close_window_control(&self, cx: &Context<Self>) -> impl IntoElement {
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
