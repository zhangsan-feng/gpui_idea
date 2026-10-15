use super::*;
use gpui_kit::component::input::Input;
impl Workspace {
    pub(super) fn render_git_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let snapshot = DATA_CONTEXT.project.git();
        let query = self.git_filter.read(cx).value().to_lowercase();
        let header = h_flex()
            .h(px(40.))
            .flex_shrink_0()
            .px_3()
            .gap_3()
            .items_center()
            .bg(color(PANEL_BG))
            .border_b_1()
            .border_color(color(BORDER))
            .child(div().font_weight(FontWeight(600.)).child("Git"))
            .child(
                h_flex()
                    .id("git-log-tab")
                    .h_full()
                    .px_2()
                    .gap_1()
                    .items_center()
                    .cursor_pointer()
                    .when(!self.git_console_selected, |tab| {
                        tab.border_b_2().border_color(color(ACCENT))
                    })
                    .child("Log")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_console_selected = false;
                        cx.notify();
                    })),
            )
            .child(
                h_flex()
                    .id("git-console-tab")
                    .h_full()
                    .px_2()
                    .items_center()
                    .cursor_pointer()
                    .when(self.git_console_selected, |tab| {
                        tab.border_b_2().border_color(color(ACCENT))
                    })
                    .child("Console")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_console_selected = true;
                        cx.notify();
                    })),
            )
            .child(div().flex_1())
            .child(
                div()
                    .id("hide-git")
                    .w(px(26.))
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|style| style.bg(color(0xe7e9ed)))
                    .child(icon(IconName::Minus, 14., MUTED))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_tool(ToolWindow::Git, cx))),
            );
        let toolbar = h_flex()
            .h(px(32.))
            .flex_shrink_0()
            .px_2()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(color(BORDER))
            .child(
                div()
                    .id("git-refresh")
                    .w(px(26.))
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|style| style.bg(color(0xe7e9ed)))
                    .child(icon(IconName::RefreshCw, 14., MUTED))
                    .on_click(cx.listener(|this, _, _, cx| this.refresh_git(cx))),
            )
            .child(
                div()
                    .w(px(240.))
                    .child(Input::new(&self.git_filter).small()),
            )
            .child(div().text_color(color(MUTED)).child(if self.git_loading {
                "正在读取 Git…".into()
            } else {
                format!("{} · {} commits", snapshot.branch, snapshot.commits.len())
            }));
        let panel = v_flex()
            .size_full()
            .min_h_0()
            .bg(color(PANEL_BG))
            .border_t_1()
            .border_color(color(BORDER))
            .child(header)
            .child(toolbar);
        if self.git_console_selected {
            return panel
                .child(
                    div()
                        .id("git-console")
                        .flex_1()
                        .min_h_0()
                        .overflow_scroll()
                        .p_3()
                        .font_family("Cascadia Mono")
                        .children(
                            snapshot
                                .status
                                .lines()
                                .map(|line| div().h(px(20.)).child(line.to_owned())),
                        ),
                )
                .into_any_element();
        }
        let details = DATA_CONTEXT
            .project
            .git_details(self.selected_git_commit.as_deref().unwrap_or_default());
        let selected = snapshot
            .commits
            .iter()
            .find(|commit| self.selected_git_commit.as_ref() == Some(&commit.hash))
            .cloned();
        panel
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .child(
                        v_flex()
                            .w(px(200.))
                            .flex_shrink_0()
                            .border_r_1()
                            .border_color(color(BORDER))
                            .p_2()
                            .gap_2()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .items_center()
                                    .child(icon(IconName::GitBranch, 14., MUTED))
                                    .child("HEAD (Current Branch)"),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .items_center()
                                    .child(icon(IconName::ChevronDown, 12., MUTED))
                                    .child("Local"),
                            )
                            .child(
                                h_flex()
                                    .h(px(26.))
                                    .pl_4()
                                    .gap_1()
                                    .items_center()
                                    .rounded_sm()
                                    .bg(color(0xe7edf8))
                                    .child(icon(IconName::GitBranch, 14., 0xc89020))
                                    .child(snapshot.branch.clone()),
                            ),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .border_r_1()
                            .border_color(color(BORDER))
                            .child(
                                h_flex()
                                    .h(px(28.))
                                    .flex_shrink_0()
                                    .px_2()
                                    .gap_2()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(color(BORDER))
                                    .text_color(color(MUTED))
                                    .child(div().flex_1().child("Commit"))
                                    .child(div().w(px(130.)).child("Author"))
                                    .child(div().w(px(88.)).child("Date")),
                            )
                            .child(
                                div()
                                    .id("git-log-list")
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_y_scroll()
                                    .when_some(snapshot.error.clone(), |list, error| {
                                        list.child(
                                            div().p_3().text_color(color(0xc42b35)).child(error),
                                        )
                                    })
                                    .children(
                                        snapshot
                                            .commits
                                            .iter()
                                            .filter(|commit| {
                                                query.is_empty()
                                                    || format!(
                                                        "{} {} {}",
                                                        commit.hash, commit.subject, commit.author
                                                    )
                                                    .to_lowercase()
                                                    .contains(&query)
                                            })
                                            .map(|commit| {
                                                let hash = commit.hash.clone();
                                                let active = self.selected_git_commit.as_ref()
                                                    == Some(&hash);
                                                h_flex()
                                                    .id(SharedString::from(format!(
                                                        "git-commit-{hash}"
                                                    )))
                                                    .h(px(28.))
                                                    .px_2()
                                                    .gap_2()
                                                    .items_center()
                                                    .cursor_pointer()
                                                    .bg(color(if active {
                                                        0xdce8ff
                                                    } else {
                                                        PANEL_BG
                                                    }))
                                                    .hover(|style| {
                                                        style.bg(color(if active {
                                                            0xdce8ff
                                                        } else {
                                                            0xedf0f5
                                                        }))
                                                    })
                                                    .child(icon(IconName::Circle, 10., 0x6eae49))
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w_0()
                                                            .truncate()
                                                            .child(commit.subject.clone()),
                                                    )
                                                    .when(!commit.references.is_empty(), |row| {
                                                        row.child(
                                                            h_flex()
                                                                .gap_1()
                                                                .items_center()
                                                                .text_color(color(0x7e8e49))
                                                                .child(icon(
                                                                    IconName::GitBranch,
                                                                    12.,
                                                                    0x9f9b3b,
                                                                ))
                                                                .child(
                                                                    commit
                                                                        .references
                                                                        .replace("HEAD -> ", ""),
                                                                ),
                                                        )
                                                    })
                                                    .child(
                                                        div()
                                                            .w(px(130.))
                                                            .truncate()
                                                            .child(commit.author.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(px(88.))
                                                            .text_color(color(MUTED))
                                                            .child(commit.date.clone()),
                                                    )
                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                        this.select_git_commit(hash.clone(), cx)
                                                    }))
                                                    .into_any_element()
                                            }),
                                    ),
                            ),
                    )
                    .child(
                        v_flex()
                            .w(px(280.))
                            .flex_shrink_0()
                            .min_h_0()
                            .child(
                                div()
                                    .id("git-details-files")
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_y_scroll()
                                    .p_3()
                                    .when(selected.is_none(), |view| {
                                        view.child(
                                            div()
                                                .text_color(color(MUTED))
                                                .child("Select commit to view changes"),
                                        )
                                    })
                                    .when_some(selected.clone(), |view, commit| {
                                        if self.git_details_loading || details.hash != commit.hash {
                                            return view.child("正在读取变更…");
                                        }
                                        view.when_some(details.error.clone(), |view, error| {
                                            view.child(
                                                div().text_color(color(0xc42b35)).child(error),
                                            )
                                        })
                                        .children(
                                            details.files.iter().map(|file| {
                                                h_flex()
                                                .h(px(24.))
                                                .gap_2()
                                                .items_center()
                                                .child(
                                                    crate::component::project_icon::project_icon(
                                                        &file.path, false,
                                                    ),
                                                )
                                                .child(
                                                    div()
                                                        .w(px(16.))
                                                        .text_color(color(
                                                            match file.status.as_str() {
                                                                "A" => 0x248a3d,
                                                                "D" => 0xc42b35,
                                                                _ => ACCENT,
                                                            },
                                                        ))
                                                        .child(file.status.clone()),
                                                )
                                                .child(div().min_w_0().truncate().child(
                                                    file.path.to_string_lossy().into_owned(),
                                                ))
                                            }),
                                        )
                                    }),
                            )
                            .child(
                                v_flex()
                                    .h(px(120.))
                                    .flex_shrink_0()
                                    .p_3()
                                    .gap_1()
                                    .border_t_1()
                                    .border_color(color(BORDER))
                                    .when_some(selected, |view, commit| {
                                        view.child(
                                            div()
                                                .h(px(20.))
                                                .flex_shrink_0()
                                                .font_weight(FontWeight(600.))
                                                .truncate()
                                                .child(commit.subject),
                                        )
                                        .child(
                                            div().h(px(20.)).flex_shrink_0().child(format!(
                                                "{} · {}",
                                                commit.author, commit.date
                                            )),
                                        )
                                        .child(
                                            div()
                                                .h(px(20.))
                                                .flex_shrink_0()
                                                .text_color(color(MUTED))
                                                .child(commit.short_hash),
                                        )
                                    })
                                    .child(
                                        div()
                                            .h(px(20.))
                                            .flex_shrink_0()
                                            .text_color(color(MUTED))
                                            .child("Commit details"),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
