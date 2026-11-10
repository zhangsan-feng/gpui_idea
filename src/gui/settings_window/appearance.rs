use super::SettingsWindow;
use gpui_kit::{component::v_flex, *};

impl SettingsWindow {
    pub(in crate::gui) fn render_appearance(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(crate::gui::component::panel::settings_header("外观"))
            .child(
                v_flex()
                    .id("appearance-settings-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .mx_4()
                    .mb_4()
                    .p_4()
                    .gap_5()
                    .border_t_1()
                    .border_color(crate::component::appearance::border(0xe5e7eb))
                    .child(self.render_themes(cx)),
            )
    }
}
