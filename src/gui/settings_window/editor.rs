use super::SettingsWindow;

use gpui_kit::{
    component::{Disableable as _, checkbox::Checkbox, v_flex},
    prelude::*,
    *,
};

const BORDER: u32 = 0xe5e7eb;
const MUTED: u32 = 0x6b7280;

impl SettingsWindow {
    pub(super) fn editor_content(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(crate::gui::component::panel::settings_header("编辑器配置"))
            .child(
                v_flex()
                    .id("editor-settings-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .mx_4()
                    .mb_4()
                    .pt_5()
                    .gap_3()
                    .border_t_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(self.text_wrap_settings(cx))
                    .child(self.formatting_settings(cx)),
            )
    }

    fn text_wrap_settings(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .bg(crate::component::appearance::background(0xffffff))
            .border_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("文本换行"))
            .child(
                Checkbox::new("editor-soft-wrap")
                    .checked(self.draft.soft_wrap)
                    .disabled(self.saving || self.system_busy)
                    .label("文本超出时自动换行")
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.draft.soft_wrap = *checked;
                        this.error = None;
                        cx.notify();
                    })),
            )
            .child(
                v_flex()
                    .pl_6()
                    .gap_2()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child("开启后，超出编辑区宽度的文本会自动折行显示，不改变文件内容。")
                    .child("关闭后保持单行显示，可横向滚动查看。")
                    .child("点击“应用”或“确定”后对所有文本标签生效。"),
            )
    }

    fn formatting_settings(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .bg(crate::component::appearance::background(0xffffff))
            .border_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("代码格式化"))
            .child(
                Checkbox::new("auto-format-on-save")
                    .checked(self.draft.auto_format)
                    .disabled(self.saving || self.system_busy)
                    .label("保存时自动格式化代码")
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.draft.auto_format = *checked;
                        this.error = None;
                        cx.notify();
                    })),
            )
            .child(
                v_flex()
                    .pl_6()
                    .gap_2()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child("开启后，手动保存和自动保存都会格式化代码。")
                    .child("需要当前语言服务支持格式化；格式化失败时仍会保存原内容。")
                    .child("点击“应用”或“确定”后生效；关闭此项仍可使用主菜单“格式化代码”或 Ctrl+Alt+L。"),
            )
    }
}
