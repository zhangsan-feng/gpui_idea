use super::*;
use crate::gui::workspace::bottom_panel::BottomPanel;

mod console;
mod header;
mod toolbar;

const RUN_SURFACE: u32 = 0xf7f8fa;
const RUN_BORDER: u32 = 0xebecf0;

impl BottomPanel {
    pub(super) fn render_run_panel(&self, cx: &mut Context<BottomPanel>) -> Div {
        v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .border_t_1()
            .border_color(crate::component::appearance::border(BORDER))
            .bg(crate::component::appearance::background(EDITOR_BG))
            .child(self.run_panel_header(cx))
            .child(self.run_toolbar(cx))
            .child(self.run_console())
    }
}
