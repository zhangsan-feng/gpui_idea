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
            ("repo", "GitHub 仓库", CreateKind::Repository),
        ] {
            tabs = tabs.child(
                Button::new(id)
                    .label(label)
                    .selected(self.kind == kind)
                    .disabled(self.busy || (kind != CreateKind::Repository && !self.is_repository))
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.select_kind(kind, window, cx)),
                    ),
            );
        }
        let help = match self.kind {
            CreateKind::LocalBranch => format!("从当前分支 {} 的 HEAD 创建，并切换到新本地分支。", self.branch),
            CreateKind::RemoteBranch => format!("将当前分支 {} 的 HEAD 发布为新的远程分支，保留当前本地分支。", self.branch),
            CreateKind::Repository => "使用 GitHub API 创建空仓库并关联当前项目的远端，不会自动推送代码；非 Git 项目会初始化本地仓库。".into(),
        };
        let mut form = v_flex()
            .gap_4()
            .child(tabs)
            .child(div().text_size(px(12.)).text_color(muted).child(help))
            .child(field(
                if self.kind == CreateKind::Repository {
                    "仓库名称"
                } else {
                    "分支名称"
                },
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
        if self.kind == CreateKind::Repository {
            form = form
                .child(field("所属组织", 82., Input::new(&self.organization).disabled(self.busy)))
                .child(field("描述", 82., Input::new(&self.description).disabled(self.busy)))
                .child(field("远端名称", 82., Input::new(&self.remote_name).disabled(self.busy)))
                .child(field("可见性", 82., h_flex().gap_2()
                    .child(Button::new("private").label("私有").selected(self.private).disabled(self.busy)
                        .on_click(cx.listener(|this, _, _, cx| { this.private = true; this.feedback = None; cx.notify(); })))
                    .child(Button::new("public").label("公开").selected(!self.private).disabled(self.busy)
                        .on_click(cx.listener(|this, _, _, cx| { this.private = false; this.feedback = None; cx.notify(); })))))
                .child(field("访问令牌", 82., Input::new(&self.token).disabled(self.busy)))
                .child(div().text_size(px(12.)).text_color(muted)
                    .child("令牌仅在本窗口内存中使用。Classic PAT 创建私有仓库需 repo 权限，公开仓库需 public_repo 或 repo 权限。"));
        }
        v_flex()
            .size_full()
            .font_family(crate::component::appearance::ui_font())
            .bg(theme.background)
            .text_color(theme.foreground)
            .text_size(px(13.))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, _| {
                if event.keystroke.key == "escape" && !this.busy {
                    window.remove_window();
                }
            }))
            .child(crate::component::operation_title_bar::render(
                "创建 Git 分支或仓库",
                !self.busy,
                cx,
            ))
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
                                    CreateKind::Repository => "创建 GitHub 仓库",
                                }
                            })
                            .disabled(
                                self.busy
                                    || self.name.read(cx).value().trim().is_empty()
                                    || self.feedback.as_ref().is_some_and(|(ok, _)| *ok)
                                    || (self.kind == CreateKind::RemoteBranch
                                        && self.remotes.is_empty())
                                    || (self.kind == CreateKind::Repository
                                        && self.token.read(cx).value().trim().is_empty()),
                            )
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
            )
    }
}
