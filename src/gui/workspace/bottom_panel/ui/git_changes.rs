use super::*;
use crate::domain::git::GitAction;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::{
    Disableable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::{Input, Textarea},
    scroll::{Scrollbar, ScrollbarMode},
};

impl BottomPanel {
    pub(super) fn render_git_changes(&self, unavailable: bool, cx: &mut Context<Self>) -> Div {
        let snapshot = DATA_CONTEXT.project.git();
        let selected = snapshot
            .changes
            .iter()
            .filter(|file| !self.state.git_excluded.contains(&file.path))
            .count();
        let all_selected = selected > 0 && selected == snapshot.changes.len();
        let conflicts = snapshot.changes.iter().any(|file| {
            file.index == 'U'
                || file.worktree == 'U'
                || (file.index == 'A' && file.worktree == 'A')
                || (file.index == 'D' && file.worktree == 'D')
        });
        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .bg(crate::component::appearance::background(EDITOR_BG))
            .border_r_1()
            .border_color(crate::component::appearance::border(BORDER))
            .child(
                h_flex()
                    .h(px(34.))
                    .flex_shrink_0()
                    .px_2()
                    .items_center()
                    .bg(crate::component::appearance::background(PANEL_BG))
                    .border_b_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(
                        Checkbox::new("git-select-all-files")
                            .checked(all_selected)
                            .label(format!("{} 个改动文件", snapshot.changes.len()))
                            .disabled(unavailable || snapshot.changes.is_empty())
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                let snapshot = DATA_CONTEXT.project.git();
                                if *checked {
                                    this.state.git_excluded.clear();
                                } else {
                                    this.state.git_excluded.extend(
                                        snapshot.changes.iter().map(|file| file.path.clone()),
                                    );
                                }
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(
                        div()
                            .id("git-working-changes")
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.state.git_changes_scroll)
                            .pr_3()
                            .when(!snapshot.is_repository, |list| {
                                list.child(div().p_3().child("当前项目不是 Git 仓库"))
                            })
                            .when(
                                snapshot.is_repository && snapshot.changes.is_empty(),
                                |list| {
                                    list.child(
                                        div()
                                            .p_3()
                                            .text_color(crate::component::appearance::foreground(
                                                MUTED,
                                            ))
                                            .child("没有改动文件"),
                                    )
                                },
                            )
                            .children(snapshot.changes.iter().map(|file| {
                                let path = file.path.clone();
                                let toggle_path = path.clone();
                                let checked = !self.state.git_excluded.contains(&path);
                                let active = self.state.git_diff_key
                                    == format!("worktree:false:{}", path.display());
                                let (symbol, color) = if file.index == 'U' || file.worktree == 'U' {
                                    (IconName::Circle, 0xc42b35)
                                } else if file.worktree == 'D'
                                    || (file.index == 'D' && file.worktree == ' ')
                                {
                                    (IconName::Minus, 0xc42b35)
                                } else if file.index == 'A' || file.index == '?' {
                                    (IconName::Plus, 0x248a3d)
                                } else {
                                    (IconName::Circle, 0xb58b00)
                                };
                                h_flex()
                                    .id(SharedString::from(format!(
                                        "git-change-{}",
                                        path.display()
                                    )))
                                    .h(px(29.))
                                    .px_2()
                                    .gap_2()
                                    .items_center()
                                    .cursor_pointer()
                                    .border_b_1()
                                    .border_color(crate::component::appearance::border(BORDER))
                                    .when(active, |row| {
                                        row.bg(crate::component::appearance::background(0xdce8ff))
                                    })
                                    .hover(|style| {
                                        style.bg(crate::component::appearance::background(0xedf0f5))
                                    })
                                    .child(
                                        Checkbox::new(SharedString::from(format!(
                                            "git-file-check-{}",
                                            path.display()
                                        )))
                                        .checked(checked)
                                        .disabled(unavailable)
                                        .on_click(
                                            cx.listener(move |this, value: &bool, _, cx| {
                                                cx.stop_propagation();
                                                if *value {
                                                    this.state.git_excluded.remove(&toggle_path);
                                                } else {
                                                    this.state
                                                        .git_excluded
                                                        .insert(toggle_path.clone());
                                                }
                                                cx.notify();
                                            }),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .child(path.to_string_lossy().into_owned()),
                                    )
                                    .child(
                                        div()
                                            .border_1()
                                            .rounded_sm()
                                            .border_color(crate::component::appearance::border(
                                                color,
                                            ))
                                            .child(icon(symbol, 10., color)),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.select_git_diff(path.clone(), false, None, cx)
                                    }))
                            })),
                    )
                    .child(
                        Scrollbar::vertical(&self.state.git_changes_scroll)
                            .mode(ScrollbarMode::Always),
                    ),
            )
            .child(
                v_flex()
                    .flex_shrink_0()
                    .p_2()
                    .gap_2()
                    .border_t_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .child(
                        Input::new(&self.state.git_summary)
                            .small()
                            .disabled(unavailable),
                    )
                    .child(
                        Textarea::new(&self.state.git_message)
                            .small()
                            .disabled(unavailable),
                    )
                    .child(
                        Button::new("git-commit-selected")
                            .primary()
                            .small()
                            .w_full()
                            .label(format!("提交 {selected} 个文件到 {}", snapshot.branch))
                            .disabled(
                                unavailable
                                    || snapshot.is_detached
                                    || conflicts
                                    || selected == 0
                                    || self.state.git_summary.read(cx).value().trim().is_empty(),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                let snapshot = DATA_CONTEXT.project.git();
                                let paths = snapshot
                                    .changes
                                    .iter()
                                    .filter(|file| !this.state.git_excluded.contains(&file.path))
                                    .map(|file| file.path.clone())
                                    .collect();
                                let summary = this.state.git_summary.read(cx).value().to_string();
                                let description =
                                    this.state.git_message.read(cx).value().to_string();
                                this.git_action(
                                    GitAction::Commit {
                                        summary,
                                        description,
                                        paths,
                                    },
                                    window,
                                    cx,
                                );
                            })),
                    ),
            )
    }
}
