use super::*;
use crate::gui::component::form::field;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    menu::DropdownMenu as _,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

impl GitCreateWindow {
    pub(super) fn render_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let mut tabs = h_flex().gap_2();
        for (id, label, kind) in [
            ("local", "本地分支", CreateKind::LocalBranch),
            ("remote", "远程分支", CreateKind::RemoteBranch),
        ] {
            tabs = tabs.child(
                Button::new(id)
                    .label(label)
                    .selected(self.kind == kind)
                    .disabled(self.busy || !self.is_repository)
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.select_kind(kind, window, cx)),
                    ),
            );
        }
        let help = match self.kind {
            CreateKind::LocalBranch => format!(
                "从当前分支 {} 的 HEAD 创建，并切换到新本地分支。",
                self.branch
            ),
            CreateKind::RemoteBranch => format!(
                "将当前分支 {} 的 HEAD 发布为新的远程分支，保留当前本地分支。",
                self.branch
            ),
        };
        let mut form = v_flex()
            .gap_4()
            .child(tabs)
            .child(
                div()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(muted)
                    .child(help),
            )
            .child(field(
                "分支名称",
                82.,
                Input::new(&self.name).disabled(self.busy),
            ));
        if self.kind == CreateKind::RemoteBranch {
            let remotes = self.remotes.clone();
            let view = cx.weak_entity();
            form = form.child(field(
                "目标远端",
                82.,
                Button::new("remote-selector")
                    .label(if self.selected_remote.is_empty() {
                        "未配置远端".to_owned()
                    } else {
                        self.selected_remote.clone()
                    })
                    .icon(gpui_kit::assets::IconName::ChevronDown)
                    .disabled(self.busy || self.remotes.is_empty())
                    .dropdown_menu(move |menu, _, _| {
                        remotes.iter().fold(menu, |menu, remote| {
                            let remote = remote.clone();
                            menu.item(crate::gui::component::menu::item(
                                remote.clone(),
                                true,
                                false,
                                &view,
                                move |this, _, cx| {
                                    this.selected_remote = remote.clone();
                                    this.feedback = None;
                                    cx.notify();
                                },
                            ))
                        })
                    }),
            ));
        }
        v_flex()
            .flex_1()
            .min_h_0()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, _| {
                if event.keystroke.key == "escape" && !this.busy {
                    window.remove_window();
                }
            }))
            .child(
                v_flex()
                    .id("git-create-form")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_5()
                    .gap_4()
                    .child(form)
                    .when_some(self.feedback.clone(), |view, (success, message)| {
                        view.child(
                            div()
                                .p_3()
                                .rounded_md()
                                .border_1()
                                .border_color(theme.border)
                                .text_color(if success {
                                    theme.foreground
                                } else {
                                    theme.danger
                                })
                                .child(message),
                        )
                    }),
            )
            .child(
                h_flex()
                    .p_4()
                    .gap_2()
                    .justify_end()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("cancel")
                            .label("关闭")
                            .disabled(self.busy)
                            .on_click(|_, window, _| window.remove_window()),
                    )
                    .child(
                        Button::new("create")
                            .primary()
                            .label(if self.busy {
                                "正在创建…"
                            } else {
                                match self.kind {
                                    CreateKind::LocalBranch => "创建并切换",
                                    CreateKind::RemoteBranch => "创建远程分支",
                                }
                            })
                            .disabled(
                                self.busy
                                    || !self.is_repository
                                    || self.name.read(cx).value().trim().is_empty()
                                    || self.feedback.as_ref().is_some_and(|(ok, _)| *ok)
                                    || (self.kind == CreateKind::RemoteBranch
                                        && self.remotes.is_empty()),
                            )
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            )
    }
}
