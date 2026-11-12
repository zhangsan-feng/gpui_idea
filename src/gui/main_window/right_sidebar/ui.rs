use super::*;
use crate::gui::main_window::right_sidebar::RightSidebar;

impl RightSidebar {
    pub(in crate::gui) fn render_right_tool_bar(&self) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .bg(crate::component::appearance::background(PANEL_BG))
            .border_l_1()
            .border_color(crate::component::appearance::border(BORDER))
    }
}
