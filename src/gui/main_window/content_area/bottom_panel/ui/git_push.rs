use super::*;
use crate::domain::git::GitAction;
use gpui_kit::component::{
    Disableable,
    button::{Button, ButtonVariants},
};

impl BottomPanel {
    pub(super) fn git_has_pending_push(&self) -> bool {
        let snapshot = DATA_CONTEXT.project.git();
        snapshot.push_remote.is_some()
            && !snapshot.is_detached
            && !snapshot.commits.is_empty()
            && (snapshot.ahead > 0 || snapshot.needs_upstream)
    }

    pub(super) fn git_push_prompt(&self, cx: &mut Context<Self>) -> Div {
        let snapshot = DATA_CONTEXT.project.git();
        let remote = snapshot.push_remote.as_deref().unwrap_or_default();
        let busy = self.state.git_busy || self.state.git_loading || self.state.git_details_loading;
        let pushing = self.state.git_commit_running && self.state.git_result_is_push;
        v_flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .p_4()
            .items_center()
            .child(
                h_flex()
                    .w_full()
                    .max_w(px(760.))
                    .p_4()
                    .gap_4()
                    .items_center()
                    .border_1()
                    .rounded_md()
                    .border_color(crate::component::appearance::border(0xbdd5ff))
                    .bg(crate::component::appearance::background(0xf0f7ff))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(crate::component::appearance::foreground(TEXT))
                                    .child(format!("推送本地提交到 {remote}")),
                            )
                            .child(
                                div()
                                    .text_color(crate::component::appearance::foreground(TEXT))
                                    .child(format!(
                                        "有 {} 个本地提交等待推送到远程。",
                                        snapshot.ahead
                                    )),
                            )
                            .child(
                                div()
                                    .text_color(crate::component::appearance::foreground(MUTED))
                                    .child(if snapshot.behind > 0 {
                                        format!(
                                            "落后远程 {} 个提交，请先拉取并处理分歧。",
                                            snapshot.behind
                                        )
                                    } else if snapshot.needs_upstream {
                                        "推送后将建立当前分支的远程跟踪关系。".into()
                                    } else {
                                        "本地提交已保存，点击右侧按钮发布到远程仓库。".into()
                                    }),
                            ),
                    )
                    .child(
                        Button::new("git-result-push-remote")
                            .primary()
                            .small()
                            .label(if pushing {
                                "正在推送…"
                            } else {
                                "推送到远程"
                            })
                            .disabled(busy)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.git_action(GitAction::Push, window, cx);
                            })),
                    ),
            )
    }
}
