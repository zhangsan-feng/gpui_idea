use super::*;
use crate::gui::main_window::content_area::bottom_panel::BottomPanel;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::Input,
    resizable::{h_resizable, resizable_panel},
    scroll::{Scrollbar, ScrollbarMode},
};

impl BottomPanel {
    pub(super) fn git_navigation(&self, cx: &mut Context<Self>) -> Div {
        let count = DATA_CONTEXT.project.git().changes.len();
        h_flex()
            .h(px(34.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(crate::component::appearance::border(BORDER))
            .children(
                [(false, format!("变更 {count}")), (true, "历史".to_owned())]
                    .into_iter()
                    .map(|(history, label)| {
                        let selected = self.state.git_history_selected == history;
                        h_flex()
                            .id(if history {
                                "git-history-tab"
                            } else {
                                "git-changes-tab"
                            })
                            .flex_1()
                            .h_full()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .when(selected, |tab| {
                                tab.border_b_2()
                                    .border_color(crate::component::appearance::border(ACCENT))
                            })
                            .hover(|style| {
                                style.bg(crate::component::appearance::background(0xedf0f5))
                            })
                            .child(label)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.state.git_show_commit_result = false;
                                this.set_git_history(history, cx)
                            }))
                    }),
            )
    }

    pub(super) fn render_git_history_list(&self, cx: &mut Context<Self>) -> Div {
        let snapshot = DATA_CONTEXT.project.git();
        let query = self.state.git_filter.read(cx).value().to_lowercase();
        let commits: Vec<_> = snapshot
            .commits
            .iter()
            .filter(|commit| {
                query.is_empty()
                    || format!(
                        "{} {} {} {}",
                        commit.hash, commit.subject, commit.author, commit.references
                    )
                    .to_lowercase()
                    .contains(&query)
            })
            .cloned()
            .collect();
        let empty = commits.is_empty();
        let list = uniform_list(
            "git-history-commits",
            commits.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .filter_map(|index| commits.get(index))
                    .map(|commit| {
                        let hash = commit.hash.clone();
                        let selected = this.state.selected_git_commit.as_ref() == Some(&hash);
                        v_flex()
                            .id(SharedString::from(format!("history-commit-{hash}")))
                            .w_full()
                            .min_w_0()
                            .h(px(54.))
                            .px_3()
                            .justify_center()
                            .gap_1()
                            .cursor_pointer()
                            .border_b_1()
                            .border_color(crate::component::appearance::border(BORDER))
                            .when(selected, |row| {
                                row.bg(crate::component::appearance::background(0xdce8ff))
                            })
                            .hover(|style| {
                                style.bg(crate::component::appearance::background(0xedf0f5))
                            })
                            .child(
                                div()
                                    .w_full()
                                    .min_w_0()
                                    .font_weight(FontWeight(600.))
                                    .truncate()
                                    .child(commit.subject.clone()),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .min_w_0()
                                    .text_color(crate::component::appearance::foreground(MUTED))
                                    .truncate()
                                    .child(format!("{} · {}", commit.author, commit.date)),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.state.git_show_commit_result = false;
                                this.select_git_commit(hash.clone(), cx)
                            }))
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::FitList)
        .track_scroll(&self.state.git_history_scroll)
        .size_full();
        v_flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .when(snapshot.commits.len() >= 100, |view| {
                view.child(
                    div()
                        .px_2()
                        .py_1()
                        .text_color(crate::component::appearance::foreground(MUTED))
                        .child("显示当前分支最近 100 条提交"),
                )
            })
            .child(
                div()
                    .p_2()
                    .flex_shrink_0()
                    .child(Input::new(&self.state.git_filter).small()),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .when(empty, |view| {
                        view.child(
                            div()
                                .p_3()
                                .text_color(crate::component::appearance::foreground(MUTED))
                                .child(if snapshot.is_repository && snapshot.commits.is_empty() {
                                    "当前分支还没有提交记录"
                                } else {
                                    "没有匹配的提交"
                                }),
                        )
                    })
                    .when(!empty, |view| {
                        view.child(list).child(
                            Scrollbar::vertical(&self.state.git_history_scroll)
                                .mode(ScrollbarMode::Always),
                        )
                    }),
            )
    }

    pub(super) fn render_git_history_details(&self, cx: &mut Context<Self>) -> Div {
        let snapshot = DATA_CONTEXT.project.git();
        let commit = snapshot
            .commits
            .iter()
            .find(|commit| self.state.selected_git_commit.as_ref() == Some(&commit.hash));
        let Some(commit) = commit else {
            return v_flex()
                .size_full()
                .min_h_0()
                .p_3()
                .text_color(crate::component::appearance::foreground(MUTED))
                .child("选择提交以查看历史改动");
        };
        let details = DATA_CONTEXT.project.git_details(&commit.hash);
        let ready = !self.state.git_details_loading && details.hash == commit.hash;
        let hash = commit.hash.clone();
        let description = details
            .message
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        let files = v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .border_r_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(
                h_flex()
                    .h(px(36.))
                    .flex_shrink_0()
                    .px_3()
                    .items_center()
                    .border_b_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(if ready {
                        format!("{} 个改动文件", details.files.len())
                    } else {
                        "正在读取改动文件…".into()
                    }),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .id("git-history-files")
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.state.git_history_files_scroll)
                            .pr_3()
                            .when_some(details.error.clone().filter(|_| ready), |list, error| {
                                list.child(
                                    div()
                                        .p_3()
                                        .text_color(crate::component::appearance::foreground(
                                            0xc42b35,
                                        ))
                                        .child(error),
                                )
                            })
                            .when(
                                ready && details.error.is_none() && details.files.is_empty(),
                                |list| list.child(div().p_3().child("此提交没有文件改动")),
                            )
                            .when(ready, |list| {
                                list.children(details.files.iter().map(|file| {
                                    let path = file.path.clone();
                                    let hash = hash.clone();
                                    let active = self.state.git_diff_key
                                        == format!("{hash}:false:{}", path.display());
                                    let color = match file.status.chars().next() {
                                        Some('A') => 0x248a3d,
                                        Some('D') => 0xc42b35,
                                        _ => ACCENT,
                                    };
                                    h_flex()
                                        .id(SharedString::from(format!(
                                            "history-file-{}",
                                            path.display()
                                        )))
                                        .h(px(30.))
                                        .px_2()
                                        .gap_2()
                                        .items_center()
                                        .cursor_pointer()
                                        .border_b_1()
                                        .border_color(crate::component::appearance::border(BORDER))
                                        .when(active, |row| {
                                            row.bg(crate::component::appearance::background(
                                                0xdce8ff,
                                            ))
                                        })
                                        .hover(|style| {
                                            style.bg(crate::component::appearance::background(
                                                0xedf0f5,
                                            ))
                                        })
                                        .when_some(file.original_path.as_ref(), |row, original| {
                                            row.child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .truncate()
                                                    .child(original.to_string_lossy().into_owned()),
                                            )
                                            .child(icon(IconName::ArrowRight, 12., MUTED))
                                        })
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .truncate()
                                                .child(path.to_string_lossy().into_owned()),
                                        )
                                        .child(
                                            div()
                                                .text_color(
                                                    crate::component::appearance::foreground(color),
                                                )
                                                .child(
                                                    file.status
                                                        .chars()
                                                        .next()
                                                        .unwrap_or('M')
                                                        .to_string(),
                                                ),
                                        )
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.state.git_show_commit_result = false;
                                            this.select_git_diff(
                                                path.clone(),
                                                false,
                                                Some(hash.clone()),
                                                cx,
                                            )
                                        }))
                                }))
                            }),
                    )
                    .child(
                        Scrollbar::vertical(&self.state.git_history_files_scroll)
                            .mode(ScrollbarMode::Always),
                    ),
            );
        let copy_hash = commit.hash.clone();
        v_flex()
            .size_full()
            .min_h_0()
            .min_w_0()
            .child(
                v_flex()
                    .flex_shrink_0()
                    .px_3()
                    .py_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(
                        div()
                            .font_weight(FontWeight(600.))
                            .truncate()
                            .child(commit.subject.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .text_color(crate::component::appearance::foreground(MUTED))
                            .child(div().truncate().child(format!(
                                "{} · {} · {}",
                                commit.author, commit.date, commit.short_hash
                            )))
                            .child(
                                Button::new("git-history-copy-hash")
                                    .ghost()
                                    .small()
                                    .icon(IconName::Copy)
                                    .on_click(move |_, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            copy_hash.clone(),
                                        ))
                                    }),
                            ),
                    )
                    .when(ready && !description.trim().is_empty(), |header| {
                        header.child(
                            div()
                                .id("git-history-message")
                                .max_h(px(100.))
                                .overflow_y_scroll()
                                .children(
                                    description.lines().map(|line| div().child(line.to_owned())),
                                ),
                        )
                    }),
            )
            .child(
                div().flex_1().min_h_0().min_w_0().child(
                    h_resizable("git-history-detail-columns")
                        .with_state(&self.state.git_history_columns)
                        .child(
                            resizable_panel()
                                .size(px(250.))
                                .size_range(px(160.)..px(700.))
                                .fixed()
                                .flex_none()
                                .child(files),
                        )
                        .child(
                            resizable_panel()
                                .size_range(px(240.)..Pixels::MAX)
                                .child(self.git_diff_view(cx)),
                        ),
                ),
            )
    }
}
