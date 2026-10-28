use super::*;
use crate::gui::workspace::right_sidebar::RightSidebar;

impl RightSidebar {
    pub(in crate::gui::workspace) fn render_right_tool_bar(&self) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .py_2()
            .bg(crate::component::appearance::background(PANEL_BG))
            .border_l_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(icon(IconName::BellDot, 17., MUTED))
            .child(icon(IconName::Workflow, 17., MUTED))
            .child(icon(IconName::Database, 17., MUTED))
            .child(icon(IconName::Code, 17., MUTED))
    }
}
