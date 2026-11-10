use super::*;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::{
    Disableable,
    button::{Button, ButtonVariants},
    scroll::{Scrollbar, ScrollbarMode},
};

impl BottomPanel {
    pub(super) fn git_content_tabs(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .h(px(36.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(crate::component::appearance::border(BORDER))
            .children([(false, "差异预览"), (true, "提交结果")].into_iter().map(
                |(result, label)| {
                    let selected = self.state.git_show_commit_result == result;
                    h_flex()
                        .id(if result {
                            "git-result-tab"
                        } else {
                            "git-preview-tab"
                        })
                        .h_full()
                        .px_3()
                        .gap_2()
                        .items_center()
                        .border_b_2()
                        .border_color(if selected {
                            crate::component::appearance::border(ACCENT)
                        } else {
                            transparent_black()
                        })
                        .text_color(crate::component::appearance::foreground(if selected {
                            TEXT
                        } else {
                            MUTED
                        }))
                        .cursor_pointer()
                        .hover(|row| row.bg(crate::component::appearance::background(0xedf0f5)))
                        .child(icon(
                            if result {
                                IconName::Terminal
                            } else {
                                IconName::File
                            },
                            14.,
                            if selected { ACCENT } else { MUTED },
                        ))
                        .child(label)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.state.git_show_commit_result = result;
                            cx.notify();
                        }))
                },
            ))
    }

    pub(super) fn git_commit_result_view(&self, cx: &mut Context<Self>) -> Div {
        let result = self.state.git_commit_result.clone();
        let title = match result.as_ref() {
            Some((true, _)) => "提交成功",
            Some((false, _)) => "提交失败",
            None if self.state.git_commit_running => "正在提交…",
            None => "暂无提交结果",
        };
        let color = match result.as_ref() {
            Some((true, _)) => 0x248a3d,
            Some((false, _)) => 0xc42b35,
            None => MUTED,
        };
        let lines = result
            .as_ref()
            .map(|(_, message)| message.lines().map(str::to_owned).collect::<Vec<_>>())
            .unwrap_or_default();
        let body = uniform_list(
            "git-commit-result-output",
            lines.len(),
            cx.processor(move |_, range: std::ops::Range<usize>, _, _| {
                range
                    .filter_map(|index| lines.get(index))
                    .map(|line| {
                        div()
                            .h(px(22.))
                            .px_4()
                            .whitespace_nowrap()
                            .font_family(crate::component::appearance::editor_font())
                            .text_size(crate::gui::component::appearance::ui_text_size(12.))
                            .child(line.clone())
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.state.git_commit_result_scroll)
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .size_full()
        .min_h_0();
        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(crate::component::appearance::background(EDITOR_BG))
            .child(
                h_flex()
                    .h(px(42.))
                    .flex_shrink_0()
                    .px_4()
                    .gap_2()
                    .items_center()
                    .border_b_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(
                        div()
                            .flex_1()
                            .text_color(crate::component::appearance::foreground(color))
                            .child(title),
                    )
                    .child(
                        Button::new("copy-git-commit-result")
                            .ghost()
                            .small()
                            .icon(IconName::Copy)
                            .label("复制结果")
                            .disabled(result.is_none())
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some((_, output)) = &this.state.git_commit_result {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        output.clone(),
                                    ));
                                }
                            })),
                    ),
            )
            .when(result.is_none(), |view| {
                view.child(
                    div()
                        .px_4()
                        .py_3()
                        .text_color(crate::component::appearance::foreground(MUTED))
                        .child(if self.state.git_commit_running {
                            "正在执行 Git 提交，请稍候。"
                        } else {
                            "提交后将在这里显示完整结果。"
                        }),
                )
            })
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(body)
                    .child(
                        Scrollbar::new(&self.state.git_commit_result_scroll)
                            .mode(ScrollbarMode::Always),
                    ),
            )
    }
}
