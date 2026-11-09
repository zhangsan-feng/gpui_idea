use super::SettingsWindow;
use crate::domain::settings::ColorTheme;
use gpui_kit::{
    component::{Disableable as _, button::Button, checkbox::Checkbox, h_flex, v_flex},
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
                    .child(self.editor_font_settings(cx))
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
                    .text_size(px(12.))
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
                    .text_size(px(12.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child("开启后，手动保存和自动保存都会格式化代码。")
                    .child("需要当前语言服务支持格式化；格式化失败时仍会保存原内容。")
                    .child("点击“应用”或“确定”后生效；关闭此项仍可使用主菜单“格式化代码”或 Ctrl+Alt+L。"),
            )
    }

    fn editor_font_settings(&self, cx: &mut Context<Self>) -> Div {
        let appearance = &self.draft.appearance;
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .bg(crate::component::appearance::background(0xffffff))
            .border_1()
            .border_color(crate::component::appearance::border(0xe5e7eb))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("字体与字号"))
            .child(self.font_picker(true, cx))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(div().w(px(110.)).child("编辑器字号"))
                    .child(
                        Button::new("font-smaller")
                            .label("−")
                            .disabled(self.saving || appearance.editor_size <= 9)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.draft.appearance.editor_size =
                                    this.draft.appearance.editor_size.saturating_sub(1).max(9);
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .w(px(40.))
                            .text_center()
                            .child(format!("{} px", appearance.editor_size)),
                    )
                    .child(
                        Button::new("font-larger")
                            .label("+")
                            .disabled(self.saving || appearance.editor_size >= 36)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.draft.appearance.editor_size =
                                    (this.draft.appearance.editor_size + 1).min(36);
                                cx.notify();
                            })),
                    ),
            )
            .child(div().font_weight(FontWeight::SEMIBOLD).child("编辑器预览"))
            .child(
                div()
                    .p_4()
                    .rounded_md()
                    .border_1()
                    .border_color(crate::component::appearance::border(0xe5e7eb))
                    .font_family(appearance.editor_font.clone())
                    .text_size(px(appearance.editor_size as f32))
                    .bg(crate::component::appearance::preview_color(
                        appearance,
                        "editor.background",
                        if appearance.theme == ColorTheme::Dark {
                            gpui_kit::rgb(0x1e1f22)
                        } else {
                            gpui_kit::rgb(0xffffff)
                        }
                        .into(),
                    ))
                    .text_color(crate::component::appearance::preview_color(
                        appearance,
                        "editor.foreground",
                        if appearance.theme == ColorTheme::Dark {
                            gpui_kit::rgb(0xdfe1e5)
                        } else {
                            gpui_kit::rgb(0x20242b)
                        }
                        .into(),
                    ))
                    .child("fn main() { println!(\"Hello, 世界 0123\"); }"),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(crate::component::appearance::foreground(0x6b7280))
                    .child("字体来自本机已安装字体。点击“应用”或“确定”后生效，重启后保留。"),
            )
    }
}
