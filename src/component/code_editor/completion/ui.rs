use super::CompletionDropdown;
use gpui_kit::{
    assets::IconName,
    component::{
        Icon, h_flex,
        scroll::{Scrollbar, ScrollbarMode},
    },
};
use gpui_kit::{prelude::*, *};
use lsp_types::CompletionItemKind;

impl CompletionDropdown {
    pub(super) fn menu(&self, width: Pixels, cx: &mut Context<Self>) -> AnyElement {
        let scrollable = self.items.len() > 9;
        let mut rows = div()
            .flex()
            .flex_col()
            .id("completion-rows")
            .size_full()
            .when(scrollable, |rows| rows.pr(Scrollbar::width()))
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        for (index, item) in self.items.iter().enumerate() {
            let keyword = item.kind == Some(CompletionItemKind::KEYWORD);
            let function = item.kind == Some(CompletionItemKind::FUNCTION);
            let prefix = item.filter_text.as_deref().unwrap_or_default();
            let matched = if item.label.starts_with(prefix) {
                prefix.len()
            } else {
                0
            };
            rows = rows.child(
                h_flex()
                    .id(index)
                    .w_full()
                    .h(px(24.))
                    .flex_shrink_0()
                    .px(px(8.))
                    .gap(px(6.))
                    .rounded(px(3.))
                    .text_color(rgb(0x202124))
                    .when(index == self.selected, |row| row.bg(rgb(0xdfe1e5)))
                    .hover(|row| row.bg(rgb(0xf0f1f3)))
                    .child(
                        Icon::new(if keyword {
                            IconName::Code
                        } else if function {
                            IconName::Square
                        } else {
                            IconName::FileBox
                        })
                        .size(px(13.))
                        .text_color(rgb(if keyword {
                            0x6b7280
                        } else {
                            0x9b51e0
                        })),
                    )
                    .child(div().max_w(px(320.)).truncate().flex_shrink_0().child(
                        StyledText::new(item.label.clone()).with_highlights([(
                            0..matched,
                            HighlightStyle {
                                color: Some(rgb(0x175fff).into()),
                                ..Default::default()
                            },
                        )]),
                    ))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_color(rgb(0x8c95a6))
                            .child(item.detail.clone().unwrap_or_default()),
                    )
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.choose(index, window, cx)),
                    ),
            );
        }
        div()
            .id("code-completion-dropdown")
            .flex()
            .flex_col()
            .occlude()
            .w(width)
            .p(px(3.))
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xb8bdc9))
            .rounded(px(7.))
            .shadow_md()
            .overflow_hidden()
            .font_family("Cascadia Mono")
            .text_size(px(13.))
            .child(
                div()
                    .relative()
                    .w_full()
                    .h(px(self.items.len().min(9) as f32 * 24.))
                    .flex_shrink_0()
                    .capture_any_mouse_down(|event, window, _| {
                        if event.button == MouseButton::Left {
                            window.prevent_default();
                        }
                    })
                    .child(rows)
                    .when(scrollable, |list| {
                        list.child(
                            Scrollbar::vertical(&self.scroll)
                                .mode(ScrollbarMode::Always)
                                .styles(|styles| {
                                    styles
                                        .thumb(|thumb| thumb.bg(rgb(0xb8bdc9)))
                                        .thumb_hover(|thumb| thumb.bg(rgb(0x8c95a6)))
                                        .thumb_active(|thumb| thumb.bg(rgb(0x6b7280)))
                                }),
                        )
                    }),
            )
            .child(
                h_flex()
                    .h(px(25.))
                    .flex_shrink_0()
                    .px(px(7.))
                    .gap(px(8.))
                    .border_t_1()
                    .border_color(rgb(0xf0f1f3))
                    .bg(rgb(0xf7f8fa))
                    .font_family("Segoe UI")
                    .text_size(px(11.))
                    .text_color(rgb(0x818999))
                    .child("↑↓ 选择   Enter 插入   Esc 关闭")
                    .child(div().flex_1())
                    .child("Ctrl+Space 提示"),
            )
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.dismiss(cx)))
            .into_any_element()
    }
}
