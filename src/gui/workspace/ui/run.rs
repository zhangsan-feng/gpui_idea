use super::*;

impl Workspace {
    pub(super) fn render_run_panel(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .size_full()
            .min_h_0()
            .border_t_1()
            .border_color(color(BORDER))
            .bg(color(EDITOR_BG))
            .child(self.run_panel_header(cx))
            .child(self.run_toolbar())
            .child(self.run_console())
    }

    fn run_panel_header(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .h(px(40.))
            .flex_shrink_0()
            .px_3()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(color(BORDER))
            .bg(color(PANEL_BG))
            .child(
                div()
                    .text_size(px(12.))
                    .font_weight(FontWeight(600.))
                    .child("Run"),
            )
            .child(
                h_flex()
                    .h_full()
                    .px_2()
                    .gap_1()
                    .items_center()
                    .border_b_2()
                    .border_color(color(ACCENT))
                    .text_size(px(12.))
                    .child(icon(IconName::CirclePlay, 14., MUTED))
                    .child("Run")
                    .child(
                        div()
                            .id("close-run-tab")
                            .w(px(18.))
                            .h(px(18.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|style| style.bg(color(0xe7e9ed)))
                            .child(icon(IconName::X, 12., MUTED))
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_run_panel(cx))),
                    ),
            )
            .child(div().flex_1())
            .child(self.console_icon(IconName::EllipsisVertical))
            .child(
                div()
                    .id("collapse-run-panel")
                    .w(px(26.))
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|style| style.bg(color(0xe7e9ed)))
                    .child(icon(IconName::Minus, 15., MUTED))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_run_panel(cx))),
            )
    }

    fn console_icon(&self, name: IconName) -> Div {
        div()
            .w(px(25.))
            .h(px(25.))
            .flex()
            .items_center()
            .justify_center()
            .text_color(color(MUTED))
            .child(icon(name, 14., MUTED))
    }

    fn run_console(&self) -> Div {
        v_flex()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .px_1()
            .py_1()
            .bg(color(EDITOR_BG))
            .font_family("Cascadia Mono")
            .text_size(px(12.))
            .text_color(color(0x243247))
    }
    fn run_toolbar(&self) -> Div {
        h_flex()
            .h(px(30.))
            .flex_shrink_0()
            .px_2()
            .gap_1()
            .bg(color(PANEL_BG))
            .border_b_1()
            .border_color(color(BORDER))
            .child(self.console_icon(IconName::Play))
            .child(self.console_icon(IconName::Square))
            .child(div().w(px(1.)).h(px(16.)).mx_1().bg(color(BORDER)))
            .child(self.console_icon(IconName::EllipsisVertical))
    }
}
