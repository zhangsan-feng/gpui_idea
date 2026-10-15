use super::*;

impl Workspace {
    pub(super) fn run_console(&self) -> Div {
        div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .pl(px(4.))
            .pt(px(3.))
            .bg(color(EDITOR_BG))
            .child(self.run_terminal.clone())
    }
}
