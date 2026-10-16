use super::*;
use gpui_kit::{
    assets::IconName,
    component::{
        Disableable as _, Icon, Sizable as _, Size,
        button::{Button, ButtonVariants as _},
        h_flex, v_flex,
    },
    prelude::*,
    *,
};

const BACKGROUND: u32 = 0xf7f8fa;
const BORDER: u32 = 0xe5e7eb;
const TEXT: u32 = 0x20242b;
const MUTED: u32 = 0x8a909e;

fn settings_icon(name: IconName, size: f32) -> Icon {
    Icon::new(name).with_size(Size::Size(px(size)))
}

impl WorkspaceSettings {
    pub(super) fn render_view(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .flex_1()
            .min_h_0()
            .bg(rgb(BACKGROUND))
            .text_size(px(13.))
            .text_color(rgb(TEXT))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_start()
                    .child(self.sidebar(cx))
                    .child(self.content()),
            )
            .child(self.footer())
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .w(px(340.))
            .h_full()
            .flex_shrink_0()
            .min_h_0()
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div().p_2().child(
                    h_flex()
                        .h(px(30.))
                        .px_2()
                        .gap_2()
                        .items_center()
                        .rounded_sm()
                        .border_1()
                        .border_color(rgb(0xc8ceda))
                        .bg(rgb(0xffffff))
                        .text_color(rgb(MUTED))
                        .child(settings_icon(IconName::Search, 14.))
                        .child(div().text_size(px(12.)).child("搜索设置")),
                ),
            )
            .child(
                v_flex()
                    .id("settings-category-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .pb_3()
                    .children(SECTIONS.iter().enumerate().map(|(index, section)| {
                        let selected = self.selected_section == index;
                        let nested = section.parent.is_some();
                        h_flex()
                            .id(("settings-category", index))
                            .h(px(25.))
                            .flex_shrink_0()
                            .items_center()
                            .gap_1()
                            .pl(px(if nested { 32. } else { 14. }))
                            .pr_3()
                            .bg(rgb(if selected { 0xd6e3ff } else { BACKGROUND }))
                            .when(!nested, |row| row.font_weight(FontWeight::SEMIBOLD))
                            .cursor_pointer()
                            .hover(move |style| {
                                style.bg(rgb(if selected { 0xd6e3ff } else { 0xebeef4 }))
                            })
                            .child(
                                div()
                                    .w(px(16.))
                                    .flex_shrink_0()
                                    .text_color(rgb(MUTED))
                                    .when(section.expandable, |slot| {
                                        slot.child(settings_icon(
                                            if index == 1 {
                                                IconName::ChevronDown
                                            } else {
                                                IconName::ChevronRight
                                            },
                                            12.,
                                        ))
                                    }),
                            )
                            .child(section.label)
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.select_section(index, cx)),
                            )
                    })),
            )
    }

    fn content(&self) -> Div {
        let section = &SECTIONS[self.selected_section];
        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                h_flex()
                    .h(px(52.))
                    .flex_shrink_0()
                    .px_4()
                    .gap_2()
                    .items_center()
                    .font_weight(FontWeight::SEMIBOLD)
                    .when_some(section.parent, |row, parent| {
                        row.child(parent)
                            .child(settings_icon(IconName::ChevronRight, 12.))
                    })
                    .child(section.label)
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_color(rgb(0xc4c8d0))
                            .child(settings_icon(IconName::ArrowLeft, 14.)),
                    )
                    .child(
                        div()
                            .text_color(rgb(0xc4c8d0))
                            .child(settings_icon(IconName::ArrowRight, 14.)),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .mx_4()
                    .mb_4()
                    .border_t_1()
                    .border_color(rgb(BORDER))
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        div()
                            .size(px(48.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_lg()
                            .bg(rgb(0xebeef4))
                            .text_color(rgb(0x9aa3b2))
                            .child(settings_icon(IconName::Settings, 24.)),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(section.label),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(MUTED))
                            .child("设置内容待添加"),
                    ),
            )
    }

    fn footer(&self) -> Div {
        h_flex()
            .h(px(52.))
            .flex_shrink_0()
            .px_4()
            .gap_2()
            .items_center()
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .size(px(22.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border_1()
                    .border_color(rgb(0xc8ceda))
                    .text_color(rgb(MUTED))
                    .child("?"),
            )
            .child(div().flex_1())
            .child(
                Button::new("settings-ok")
                    .primary()
                    .w(px(76.))
                    .label("确定")
                    .on_click(|_, window, _| window.remove_window()),
            )
            .child(
                Button::new("settings-cancel")
                    .w(px(76.))
                    .label("取消")
                    .on_click(|_, window, _| window.remove_window()),
            )
            .child(
                Button::new("settings-apply")
                    .w(px(76.))
                    .label("应用")
                    .disabled(true),
            )
    }
}
