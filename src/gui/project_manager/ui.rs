use super::*;
use crate::{data_context::DATA_CONTEXT, domain::history::OpenKind};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::Input,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

impl ProjectManager {
    pub(super) fn render_view(&self, cx: &mut Context<Self>) -> Div {
        if self.mode == ProjectWindowKind::Create {
            return self.render_create(cx);
        }
        let theme = cx.theme();
        let mut view = v_flex()
            .size_full()
            .p_5()
            .gap_4()
            .bg(theme.background)
            .text_color(theme.foreground);
        if self.mode != ProjectWindowKind::Manage {
            view = view.child(
                v_flex().gap_2().child("项目目录").child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Input::new(&self.directory).disabled(self.busy)),
                        )
                        .child(
                            Button::new("choose-project-directory")
                                .label("浏览…")
                                .disabled(self.busy)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.choose_directory(window, cx)
                                })),
                        ),
                ),
            );
        }
        {
            let history = DATA_CONTEXT.history.snapshot();
            let projects: Vec<_> = history
                .entries
                .iter()
                .filter(|entry| entry.kind == OpenKind::Project)
                .collect();
            let mut list = v_flex()
                .id("recent-project-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .gap_2();
            for (index, project) in projects.iter().enumerate() {
                let path = project.path.clone();
                let open_path = path.clone();
                let remove_path = path.clone();
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                list = list.child(
                    h_flex()
                        .id(("recent-project-row", index))
                        .w_full()
                        .p_3()
                        .gap_3()
                        .rounded_md()
                        .border_1()
                        .border_color(theme.border)
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(div().font_weight(FontWeight(600.)).child(name))
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(12.))
                                        .text_color(theme.muted_foreground)
                                        .child(path.display().to_string()),
                                ),
                        )
                        .child(
                            Button::new(("open-recent-project", index))
                                .small()
                                .label("打开")
                                .icon(IconName::FolderOpen)
                                .disabled(self.busy)
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_project(open_path.clone(), window, cx)
                                })),
                        )
                        .when(self.mode == ProjectWindowKind::Manage, |row| {
                            row.child(
                                Button::new(("forget-project", index))
                                    .ghost()
                                    .small()
                                    .label("移除记录")
                                    .disabled(self.busy)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.forget_project(remove_path.clone(), cx)
                                    })),
                            )
                        }),
                );
            }
            if projects.is_empty() {
                list = list.child(
                    div()
                        .text_color(theme.muted_foreground)
                        .child("暂无最近打开的项目"),
                );
            }
            view = view
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight(600.))
                        .child("最近打开的项目"),
                )
                .child(list);
        }
        view = view
            .when_some(self.error.clone(), |view, error| {
                view.child(div().text_size(px(12.)).text_color(theme.red).child(error))
            })
            .when_some(self.status.clone(), |view, status| {
                view.child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.muted_foreground)
                        .child(status),
                )
            });
        let mut footer = h_flex().justify_end().gap_2().child(
            Button::new("close-project-window")
                .label("关闭")
                .disabled(self.busy)
                .on_click(|_, window, _| window.remove_window()),
        );
        if self.mode != ProjectWindowKind::Manage {
            footer = footer.child(
                Button::new("submit-project-window")
                    .primary()
                    .label(if self.busy {
                        "处理中…"
                    } else {
                        "打开项目"
                    })
                    .disabled(self.busy)
                    .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
            );
        }
        view.child(footer)
    }
}
