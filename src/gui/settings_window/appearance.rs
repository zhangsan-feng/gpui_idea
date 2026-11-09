use super::SettingsWindow;
use gpui_kit::{
    component::{
        Disableable as _,
        button::Button,
        h_flex,
        menu::{DropdownMenu, PopupMenuItem},
        v_flex,
    },
    prelude::*,
    *,
};

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
                    .child(self.render_themes(cx))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("字体"))
                    .child(self.font_picker(false, cx))
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(crate::component::appearance::foreground(0x6b7280))
                            .child(
                                "字体来自本机已安装字体。点击“应用”或“确定”后生效，重启后保留。",
                            ),
                    ),
            )
    }

    pub(super) fn font_picker(&self, editor: bool, cx: &mut Context<Self>) -> Div {
        let fonts = self.fonts.clone();
        let view = cx.entity().downgrade();
        let selected = if editor {
            &self.draft.appearance.editor_font
        } else {
            &self.draft.appearance.ui_font
        };
        h_flex()
            .gap_3()
            .items_center()
            .child(div().w(px(110.)).child(if editor {
                "编辑器字体"
            } else {
                "界面字体"
            }))
            .child(
                Button::new(if editor { "editor-font" } else { "ui-font" })
                    .label(selected.clone())
                    .disabled(self.saving)
                    .w(px(280.))
                    .dropdown_menu(move |mut menu, _, _| {
                        menu = menu.min_w(px(280.)).max_h(px(320.)).scrollable(true);
                        for font in &fonts {
                            let font = font.clone();
                            let view = view.clone();
                            menu = menu.item(PopupMenuItem::new(font.clone()).on_click(
                                move |_, _, cx| {
                                    let _ = view.update(cx, |this, cx| {
                                        if editor {
                                            this.draft.appearance.editor_font = font.clone();
                                        } else {
                                            this.draft.appearance.ui_font = font.clone();
                                        }
                                        this.error = None;
                                        cx.notify();
                                    });
                                },
                            ));
                        }
                        menu
                    }),
            )
    }
}
