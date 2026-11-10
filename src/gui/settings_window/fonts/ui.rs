use super::super::SettingsWindow;
use crate::domain::settings::ColorTheme;
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
    pub(in crate::gui::settings_window) fn render_fonts(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(crate::gui::component::panel::settings_header("字体"))
            .child(
                v_flex()
                    .id("font-settings-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .mx_4()
                    .mb_4()
                    .pt_5()
                    .gap_3()
                    .border_t_1()
                    .border_color(crate::component::appearance::border(0xe5e7eb))
                    .child(self.ui_font_settings(cx))
                    .child(self.editor_font_settings(cx)),
            )
    }

    fn ui_font_settings(&self, cx: &mut Context<Self>) -> Div {
        let appearance = &self.draft.appearance;
        crate::gui::component::panel::card(0xe5e7eb)
            .child(div().font_weight(FontWeight::SEMIBOLD).child("界面字体"))
            .child(self.font_picker(false, cx))
            .child(self.font_size_picker(false, cx))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("界面预览"))
            .child(
                div()
                    .p_4()
                    .rounded_md()
                    .font_family(appearance.ui_font.clone())
                    .text_size(px(appearance.ui_size as f32))
                    .child("设置 · 项目 · 文件  Settings · Project · File  0123456789"),
            )
    }

    fn editor_font_settings(&self, cx: &mut Context<Self>) -> Div {
        let appearance = &self.draft.appearance;
        crate::gui::component::panel::card(0xe5e7eb)
            .child(div().font_weight(FontWeight::SEMIBOLD).child("编辑器字体"))
            .child(self.font_picker(true, cx))
            .child(self.font_size_picker(true, cx))
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
                        rgb(if appearance.theme == ColorTheme::Dark {
                            0x1e1f22
                        } else {
                            0xffffff
                        })
                        .into(),
                    ))
                    .text_color(crate::component::appearance::preview_color(
                        appearance,
                        "editor.foreground",
                        rgb(if appearance.theme == ColorTheme::Dark {
                            0xdfe1e5
                        } else {
                            0x20242b
                        })
                        .into(),
                    ))
                    .child("fn main() { println!(\"Hello, 世界 0123\"); }"),
            )
    }

    fn font_size_picker(&self, editor: bool, cx: &mut Context<Self>) -> Div {
        let size = if editor {
            self.draft.appearance.editor_size
        } else {
            self.draft.appearance.ui_size
        };
        let (minimum, maximum) = if editor { (9, 36) } else { (10, 24) };
        let busy = self.saving;
        h_flex()
            .gap_3()
            .items_center()
            .child(div().w(px(110.)).child(if editor {
                "编辑器字号"
            } else {
                "窗口字号"
            }))
            .child(
                Button::new(if editor {
                    "editor-font-smaller"
                } else {
                    "ui-font-smaller"
                })
                .label("−")
                .disabled(busy || size <= minimum)
                .on_click(cx.listener(move |this, _, _, cx| {
                    let size = if editor {
                        &mut this.draft.appearance.editor_size
                    } else {
                        &mut this.draft.appearance.ui_size
                    };
                    *size = size.saturating_sub(1).max(minimum);
                    this.error = None;
                    cx.notify();
                })),
            )
            .child(
                div()
                    .min_w(px(44.))
                    .text_center()
                    .child(format!("{size} px")),
            )
            .child(
                Button::new(if editor {
                    "editor-font-larger"
                } else {
                    "ui-font-larger"
                })
                .label("+")
                .disabled(busy || size >= maximum)
                .on_click(cx.listener(move |this, _, _, cx| {
                    let size = if editor {
                        &mut this.draft.appearance.editor_size
                    } else {
                        &mut this.draft.appearance.ui_size
                    };
                    *size = size.saturating_add(1).min(maximum);
                    this.error = None;
                    cx.notify();
                })),
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
            .flex_wrap()
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
            .child(
                Button::new(if editor {
                    "reset-editor-fonts"
                } else {
                    "reset-ui-fonts"
                })
                .label("恢复默认")
                .ml(px(64.))
                .disabled(self.saving)
                .on_click(cx.listener(move |this, _, _, cx| this.reset_font(editor, cx))),
            )
    }
}
