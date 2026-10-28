use super::*;
use crate::gui::workspace::bottom_panel::BottomPanel;

impl BottomPanel {
    pub(super) fn run_console(&self) -> Div {
        div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .pl(px(4.))
            .pt(px(3.))
            .bg(crate::component::appearance::background(EDITOR_BG))
            .child(self.state.run_terminal.clone())
    }
}
