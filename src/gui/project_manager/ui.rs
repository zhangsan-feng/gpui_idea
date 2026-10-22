use super::*;
use crate::{data_context::DATA_CONTEXT, domain::history::OpenKind};
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, Size,
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
            .bg(rgb(0xf7f8fa))
            .text_size(px(13.))
            .text_color(rgb(0x20242b));
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
                .gap_3();
            for (index, project) in projects.iter().enumerate() {
                let path = project.path.clone();
                let open_path = path.clone();
                let remove_path = path.clone();
                let display_path = display_project_path(&path);
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| display_path.clone());
                list = list.child(
                    h_flex()
                        .id(("recent-project-row", index))
                        .w_full()
                        .min_h(px(84.))
                        .items_center()
                        .p_4()
                        .gap_3()
                        .rounded_lg()
                        .bg(rgb(0xffffff))
                        .border_1()
                        .border_color(rgb(0xe5e7eb))
                        .when(!self.busy, |row| {
                            row.hover(|style| style.border_color(rgb(0xbfdbfe)).bg(rgb(0xfcfdff)))
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(40.))
                                .flex_shrink_0()
                                .rounded_lg()
                                .bg(rgb(0xeff6ff))
                                .text_color(rgb(0x3b82f6))
                                .child(Icon::new(IconName::Folder).with_size(Size::Size(px(20.)))),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(14.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child(name),
                                )
                                .child(
                                    div()
                                        .truncate()
                                        .text_size(px(12.))
                                        .text_color(rgb(0x7b8494))
                                        .child(display_path),
                                ),
                        )
                        .child(
                            Button::new(("open-recent-project", index))
                                .small()
                                .w(px(72.))
                                .h(px(30.))
                                .rounded_md()
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
                                    .icon(IconName::Trash)
                                    .tooltip("移除记录，保留项目文件")
                                    .text_color(rgb(0x9ca3af))
                                    .hover(|style| {
                                        style.bg(rgb(0xfef2f2)).text_color(rgb(0xdc2626))
                                    })
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
                    v_flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .gap_3()
                        .py_5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size(px(64.))
                                .rounded_lg()
                                .bg(rgb(0xeef1f6))
                                .text_color(rgb(0x94a3b8))
                                .child(
                                    Icon::new(IconName::FolderOpen).with_size(Size::Size(px(28.))),
                                ),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("暂无最近项目"),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(0x7b8494))
                                .child("打开项目文件夹后会显示在这里"),
                        ),
                );
            }
            view = view
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(17.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .child("最近项目"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(rgb(0xe9edf3))
                                        .text_size(px(11.))
                                        .text_color(rgb(0x64748b))
                                        .child(projects.len().to_string()),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(0x7b8494))
                                .child("快速返回最近使用的项目"),
                        ),
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
        let mut footer = h_flex()
            .items_center()
            .justify_end()
            .gap_2()
            .pt_4()
            .border_t_1()
            .border_color(rgb(0xe5e7eb))
            .child(
                Button::new("close-project-window")
                    .w(px(76.))
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

fn display_project_path(path: &std::path::Path) -> String {
    let path = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(network) = path.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{network}")
        } else {
            path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned()
        }
    }
    #[cfg(not(windows))]
    {
        path.into_owned()
    }
}
