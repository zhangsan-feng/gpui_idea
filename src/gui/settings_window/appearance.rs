use super::SettingsWindow;
use crate::domain::settings::ColorTheme;
use gpui_kit::{
    component::{
        Disableable as _, Selectable as _,
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
        let appearance = &self.draft.appearance;
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
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("主题"))
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                Button::new("light-theme")
                                    .label("浅色")
                                    .selected(appearance.theme == ColorTheme::Light)
                                    .disabled(self.saving)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.draft.appearance.theme = ColorTheme::Light;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("dark-theme")
                                    .label("深色")
                                    .selected(appearance.theme == ColorTheme::Dark)
                                    .disabled(self.saving)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.draft.appearance.theme = ColorTheme::Dark;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("字体"))
                    .child(self.font_picker(false, cx))
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
                                        this.draft.appearance.editor_size = this
                                            .draft
                                            .appearance
                                            .editor_size
                                            .saturating_sub(1)
                                            .max(9);
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
                            .bg(if appearance.theme == ColorTheme::Dark {
                                gpui_kit::rgb(0x1e1f22)
                            } else {
                                gpui_kit::rgb(0xffffff)
                            })
                            .text_color(if appearance.theme == ColorTheme::Dark {
                                gpui_kit::rgb(0xdfe1e5)
                            } else {
                                gpui_kit::rgb(0x20242b)
                            })
                            .child("fn main() { println!(\"Hello, 世界 0123\"); }"),
                    )
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

    fn font_picker(&self, editor: bool, cx: &mut Context<Self>) -> Div {
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
