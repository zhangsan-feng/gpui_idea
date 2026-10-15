use super::*;

impl Workspace {
    pub(in crate::gui::workspace::ui) fn render_right_tool_bar(&self) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .py_2()
            .bg(color(PANEL_BG))
            .border_l_1()
            .border_color(color(BORDER))
            .child(icon(IconName::BellDot, 17., MUTED))
            .child(icon(IconName::Workflow, 17., MUTED))
            .child(icon(IconName::Database, 17., MUTED))
            .child(icon(IconName::Code, 17., MUTED))
    }
}
