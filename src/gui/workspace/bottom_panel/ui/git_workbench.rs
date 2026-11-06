use super::*;
use crate::domain::git::GitAction;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::Disableable;
use gpui_kit::component::menu::DropdownMenu;
use gpui_kit::component::scroll::{Scrollbar, ScrollbarMode};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    resizable::{h_resizable, resizable_panel},
};

impl BottomPanel {
    pub(super) fn render_git_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let snapshot = DATA_CONTEXT.project.git();
        let busy = self.state.git_busy || self.state.git_loading || self.state.git_details_loading;
        let unavailable = busy || !snapshot.is_repository;
        let toolbar = h_flex()
            .h(px(42.))
            .flex_shrink_0()
            .px_3()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(icon(IconName::GitBranch, 16., ACCENT))
            .child(self.git_branch_selector(unavailable, cx))
            .child(
                Button::new("git-checkout-remote")
                    .small()
                    .icon(IconName::RefreshCw)
                    .label("从远程拉取")
                    .disabled(unavailable || snapshot.is_detached)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.git_action(GitAction::UpdateFromRemote, window, cx)
                    })),
            )
            .child(div().flex_1())
            .when(busy, |bar| {
                bar.child(
                    div()
                        .text_color(crate::component::appearance::foreground(MUTED))
                        .child("正在处理…"),
                )
            })
            .child(
                Button::new("git-workbench-close")
                    .ghost()
                    .small()
                    .icon(IconName::X)
                    .on_click(
                        cx.listener(|this, _, _window, cx| this.toggle_tool(ToolWindow::Git, cx)),
                    ),
            );
        let changes = v_flex()
            .size_full()
            .min_h_0()
            .child(self.git_navigation(cx))
            .child(if self.state.git_history_selected {
                self.render_git_history_list(cx).into_any_element()
            } else {
                self.render_git_changes(unavailable, cx).into_any_element()
            });
        let detail = if self.state.git_history_selected {
            self.render_git_history_details(cx)
        } else {
            self.git_diff_view(cx)
        };
        let upper = v_flex()
            .size_full()
            .min_h_0()
            .child(toolbar)
            .when_some(snapshot.error.clone(), |view, error| {
                view.child(
                    div()
                        .px_3()
                        .py_1()
                        .text_color(crate::component::appearance::foreground(0xc42b35))
                        .child(error),
                )
            })
            .when_some(
                self.state.git_feedback.clone(),
                |view, (success, message)| {
                    view.child(
                        div()
                            .id("git-operation-result")
                            .max_h(px(70.))
                            .overflow_y_scroll()
                            .px_3()
                            .py_1()
                            .text_color(crate::component::appearance::foreground(if success {
                                0x248a3d
                            } else {
                                0xc42b35
                            }))
                            .child(message),
                    )
                },
            )
            .child(
                div().flex_1().min_h_0().min_w_0().child(
                    h_resizable("git-working-columns")
                        .with_state(&self.state.git_columns)
                        .child(
                            resizable_panel()
                                .size(px(310.))
                                .size_range(px(230.)..px(700.))
                                .fixed()
                                .flex_none()
                                .child(changes),
                        )
                        .child(
                            resizable_panel()
                                .size_range(px(240.)..Pixels::MAX)
                                .child(detail),
                        ),
                ),
            );
        upper.into_any_element()
    }

    fn git_branch_selector(&self, disabled: bool, cx: &Context<Self>) -> impl IntoElement {
        let snapshot = DATA_CONTEXT.project.git();
        let view = cx.entity().downgrade();
        Button::new("git-commit-branch")
            .ghost()
            .small()
            .label(format!("Git · {}", snapshot.branch))
            .icon(IconName::ChevronDown)
            .disabled(disabled)
            .dropdown_menu(move |menu, _, _| {
                snapshot.branches.iter().fold(
                    menu.label("选择提交分支（切换当前分支）"),
                    |menu, branch| {
                        let branch = branch.clone();
                        let current = snapshot.branch == branch;
                        menu.item(crate::gui::component::menu::item(
                            branch.clone(),
                            !disabled && !current,
                            current,
                            &view,
                            move |this, window, cx| {
                                this.git_action(GitAction::Switch(branch.clone()), window, cx)
                            },
                        ))
                    },
                )
            })
    }

    pub(super) fn git_diff_view(&self, cx: &mut Context<Self>) -> Div {
        let snapshot = DATA_CONTEXT.project.git_diff();
        let message = if self.state.git_diff_loading {
            Some("正在读取差异…".to_owned())
        } else if self.state.git_diff_key.is_empty() {
            Some("选择变更文件或提交中的文件以查看差异".to_owned())
        } else if snapshot.key != self.state.git_diff_key {
            Some("差异已过期，请重新选择文件".to_owned())
        } else {
            snapshot.error.clone()
        };
        let count = if message.is_some() {
            1
        } else {
            snapshot.lines.len()
        };
        let body = uniform_list(
            "git-diff-content",
            count,
            cx.processor(move |_this, range: std::ops::Range<usize>, _, _cx| {
                range
                    .filter_map(|index| {
                        let line = message.as_ref().or_else(|| snapshot.lines.get(index))?;
                        Some(
                            div()
                                .h(px(21.))
                                .px_3()
                                .whitespace_nowrap()
                                .font_family(crate::component::appearance::editor_font())
                                .when(line.starts_with('+'), |row| {
                                    row.bg(crate::component::appearance::background(0xe6f4e9))
                                        .text_color(crate::component::appearance::foreground(
                                            0x248a3d,
                                        ))
                                })
                                .when(line.starts_with('-'), |row| {
                                    row.bg(crate::component::appearance::background(0xfce8e8))
                                        .text_color(crate::component::appearance::foreground(
                                            0xc42b35,
                                        ))
                                })
                                .when(line.starts_with("@@"), |row| {
                                    row.text_color(crate::component::appearance::foreground(ACCENT))
                                })
                                .child(line.clone()),
                        )
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .track_scroll(&self.state.git_diff_scroll)
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
        .size_full()
        .min_h_0();
        v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                h_flex()
                    .h(px(36.))
                    .flex_shrink_0()
                    .px_3()
                    .gap_2()
                    .items_center()
                    .border_b_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(icon(IconName::File, 14., MUTED))
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .child(if self.state.git_diff_key.is_empty() {
                                "差异预览".into()
                            } else {
                                self.state.git_diff_title.clone()
                            }),
                    )
                    .child(
                        div()
                            .text_color(crate::component::appearance::foreground(MUTED))
                            .child("统一差异"),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(body)
                    .child(Scrollbar::new(&self.state.git_diff_scroll).mode(ScrollbarMode::Always)),
            )
    }
}
