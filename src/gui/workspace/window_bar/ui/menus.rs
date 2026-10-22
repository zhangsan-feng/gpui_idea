use super::*;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};

impl WindowBar {
    pub(in crate::gui::workspace) fn toolbar_item(
        label: impl Into<SharedString>,
        action: ToolbarAction,
        enabled: bool,
        checked: bool,
        view: &WeakEntity<WindowBar>,
    ) -> PopupMenuItem {
        let view = view.clone();
        PopupMenuItem::new(label)
            .disabled(!enabled)
            .checked(checked)
            .on_click(move |_, window, cx| {
                if enabled {
                    let _ = view.update(cx, |this, cx| {
                        this.toolbar_action(action.clone(), window, cx)
                    });
                }
            })
    }

    pub(super) fn main_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        self.toolbar_button("main-menu", cx)
            .w(px(40.))
            .tooltip("主菜单")
            .child(icon(IconName::Menu, 18., 0xb9bdc5))
            .dropdown_menu(move |menu, _, _| {
                menu.item(Self::toolbar_item(
                    "新建项目…",
                    ToolbarAction::CreateProject,
                    true,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "打开项目…",
                    ToolbarAction::OpenProject,
                    true,
                    false,
                    &view,
                ))
            })
    }

    pub(super) fn project_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let root = DATA_CONTEXT.project_root();
        let name = if DATA_CONTEXT.has_project() {
            root.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            "打开项目".to_owned()
        };
        let initials: String = name
            .split(['_', '-', ' '])
            .filter_map(|part| part.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect();
        self.toolbar_button("project-menu", cx)
            .max_w(px(190.))
            .tooltip(if DATA_CONTEXT.has_project() {
                display_project_path(root)
            } else {
                "最近项目与项目管理".to_owned()
            })
            .child(
                div()
                    .w(px(20.))
                    .h(px(20.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .bg(color(0x159cab))
                    .text_color(color(0xffffff))
                    .text_size(px(11.))
                    .when(DATA_CONTEXT.has_project(), |tile| tile.child(initials))
                    .when(!DATA_CONTEXT.has_project(), |tile| {
                        tile.child(icon(IconName::FolderOpen, 13., 0xffffff))
                    }),
            )
            .child(div().min_w_0().truncate().child(name))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5))
            .dropdown_menu(move |menu, _, _| {
                let history = DATA_CONTEXT.history.snapshot();
                let total = history
                    .entries
                    .iter()
                    .filter(|entry| entry.kind == crate::domain::history::OpenKind::Project)
                    .count();
                let mut menu = menu
                    .min_w(px(328.))
                    .max_w(px(328.))
                    .max_h(px(520.))
                    .scrollable(true)
                    .item(
                        PopupMenuItem::element(move |_, _| {
                            project_menu_row()
                                .h(px(32.))
                                .px_1()
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(color(0x64748b))
                                        .child("最近项目"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(color(0xf1f5f9))
                                        .text_size(px(11.))
                                        .text_color(color(0x64748b))
                                        .child(total.to_string()),
                                )
                        })
                        .disabled(true),
                    );
                let mut count = 0;
                for entry in history
                    .entries
                    .iter()
                    .filter(|entry| entry.kind == crate::domain::history::OpenKind::Project)
                    .take(15)
                {
                    let path = entry.path.clone();
                    let name = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| display_project_path(&path));
                    let display_path = display_project_path(&path);
                    let current = DATA_CONTEXT.has_project()
                        && crate::domain::history::project_identity(&path)
                            == crate::domain::history::project_identity(
                                DATA_CONTEXT.project_root(),
                            );
                    let open_view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::element(move |_, _| {
                            project_menu_row()
                                .h(px(56.))
                                .px_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(32.))
                                        .flex_shrink_0()
                                        .rounded_md()
                                        .bg(color(0xeff6ff))
                                        .child(icon(IconName::Folder, 17., 0x3b82f6)),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .gap_1()
                                        .child(
                                            h_flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .min_w_0()
                                                        .truncate()
                                                        .font_weight(FontWeight::SEMIBOLD)
                                                        .text_color(color(0x20242b))
                                                        .child(name.clone()),
                                                )
                                                .when(current, |row| {
                                                    row.child(
                                                        div()
                                                            .flex_shrink_0()
                                                            .px_1()
                                                            .rounded_sm()
                                                            .bg(color(0xeff6ff))
                                                            .text_size(px(10.))
                                                            .text_color(color(0x2563eb))
                                                            .child("当前"),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .truncate()
                                                .text_size(px(11.))
                                                .text_color(color(0x7b8494))
                                                .child(display_path.clone()),
                                        ),
                                )
                        })
                        .on_click(move |_, window, cx| {
                            let _ = open_view.update(cx, |this, cx| {
                                this.toolbar_action(
                                    ToolbarAction::OpenRecentProject(path.clone()),
                                    window,
                                    cx,
                                )
                            });
                        }),
                    );
                    count += 1;
                }
                if count == 0 {
                    menu = menu.item(
                        PopupMenuItem::element(|_, _| {
                            project_menu_row()
                                .h(px(48.))
                                .px_1()
                                .child(icon(IconName::FolderOpen, 18., 0x94a3b8))
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(color(0x7b8494))
                                        .child("暂无最近项目"),
                                )
                        })
                        .disabled(true),
                    );
                }
                let manage_view = view.clone();
                menu.separator().item(
                    PopupMenuItem::element(|_, _| {
                        project_menu_row()
                            .h(px(36.))
                            .px_1()
                            .child(icon(IconName::Settings, 16., 0x64748b))
                            .child(div().flex_1().child("管理项目"))
                            .child(icon(IconName::ChevronRight, 13., 0x94a3b8))
                    })
                    .on_click(move |_, window, cx| {
                        let _ = manage_view.update(cx, |this, cx| {
                            this.toolbar_action(ToolbarAction::ManageProjects, window, cx)
                        });
                    }),
                )
            })
    }

    pub(super) fn branch_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let snapshot = DATA_CONTEXT.project.git();
        let available =
            snapshot.is_repository && !snapshot.is_detached && !snapshot.branch.is_empty();
        let label = snapshot.branch.clone();
        let tooltip = snapshot
            .error
            .clone()
            .unwrap_or_else(|| format!("Git · {label}"));
        self.toolbar_button("branch-menu", cx)
            .max_w(px(160.))
            .tooltip(tooltip)
            .child(icon(IconName::GitBranch, 15., 0xb9bdc5))
            .child(div().min_w_0().truncate().child(label))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5))
            .dropdown_menu(move |menu, _, _| {
                menu.label(if snapshot.is_detached {
                    format!("当前提交：{}", snapshot.branch)
                } else if available {
                    format!("当前分支：{}", snapshot.branch)
                } else {
                    "当前项目没有可用的 Git 分支".into()
                })
                .item(Self::toolbar_item(
                    "打开 Git Log",
                    ToolbarAction::Tool(ToolWindow::Git),
                    true,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "刷新分支和工作区状态",
                    ToolbarAction::RefreshGit,
                    true,
                    false,
                    &view,
                ))
                .item(Self::toolbar_item(
                    "复制当前分支名",
                    ToolbarAction::CopyBranch,
                    available,
                    false,
                    &view,
                ))
            })
    }

    pub(super) fn more_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        self.toolbar_button("more-menu", cx)
            .w(px(30.))
            .tooltip("更多工具窗口")
            .child(icon(IconName::EllipsisVertical, 16., 0xb9bdc5))
            .dropdown_menu(move |menu, _, _| {
                [
                    ("Run", ToolWindow::Run),
                    ("Terminal", ToolWindow::Terminal),
                    ("Git", ToolWindow::Git),
                    ("Statistics", ToolWindow::Statistics),
                ]
                .into_iter()
                .fold(menu.label("Tool Windows"), |menu, (label, tool)| {
                    menu.item(Self::toolbar_item(
                        label,
                        ToolbarAction::Tool(tool),
                        true,
                        false,
                        &view,
                    ))
                })
            })
    }

    pub(super) fn settings_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        self.toolbar_button("settings-menu", cx)
            .w(px(30.))
            .tooltip("工作区配置")
            .child(icon(IconName::Settings, 18., 0xb9bdc5))
            .on_click(cx.listener(|this, _, window, cx| this.open_settings(window, cx)))
    }
}

fn project_menu_row() -> Div {
    h_flex()
        .w(px(300.))
        .min_w_0()
        .items_center()
        .gap_3()
        .font_family("Microsoft YaHei UI")
        .text_size(px(13.))
        .text_color(color(0x20242b))
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
