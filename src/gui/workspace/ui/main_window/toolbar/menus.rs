use super::*;
use crate::gui::workspace::ToolbarAction;
use gpui_kit::component::menu::{DropdownMenu, PopupMenuItem};

impl Workspace {
    pub(in crate::gui::workspace::ui) fn toolbar_item(
        label: impl Into<SharedString>,
        action: ToolbarAction,
        enabled: bool,
        checked: bool,
        view: &WeakEntity<Self>,
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

    pub(super) fn main_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let has_file = !self.open_files.is_empty();
        self.toolbar_button("main-menu", cx)
            .w(px(40.))
            .tooltip("主菜单")
            .child(icon(IconName::Menu, 18., 0xb9bdc5))
            .dropdown_menu(move |menu, _, _| {
                menu.label("项目")
                    .item(Self::toolbar_item(
                        "打开新项目…",
                        ToolbarAction::OpenProject,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "创建新项目…",
                        ToolbarAction::CreateProject,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "管理项目…",
                        ToolbarAction::ManageProjects,
                        true,
                        false,
                        &view,
                    ))
                    .separator()
                    .item(Self::toolbar_item(
                        "编辑运行配置…",
                        ToolbarAction::EditRun,
                        true,
                        false,
                        &view,
                    ))
                    .separator()
                    .label("文件")
                    .item(Self::toolbar_item(
                        "复制项目路径",
                        ToolbarAction::CopyProjectPath,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "保存当前文件    Ctrl+S",
                        ToolbarAction::Save,
                        has_file,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "关闭所有编辑器标签",
                        ToolbarAction::CloseTabs,
                        has_file,
                        false,
                        &view,
                    ))
                    .separator()
                    .label("View")
                    .item(Self::toolbar_item(
                        "代码内容搜索…",
                        ToolbarAction::Search,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "Project",
                        ToolbarAction::ToggleProject,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "Terminal",
                        ToolbarAction::Tool(ToolWindow::Terminal),
                        true,
                        false,
                        &view,
                    ))
                    .separator()
                    .item(Self::toolbar_item(
                        "退出",
                        ToolbarAction::CloseWindow,
                        true,
                        false,
                        &view,
                    ))
            })
    }

    pub(super) fn project_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let root = DATA_CONTEXT.project_root();
        let name = root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let initials: String = name
            .split(['_', '-', ' '])
            .filter_map(|part| part.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect();
        self.toolbar_button("project-menu", cx)
            .max_w(px(190.))
            .tooltip(root.display().to_string())
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
                    .child(initials),
            )
            .child(div().min_w_0().truncate().child(name))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5))
            .dropdown_menu(move |menu, _, _| {
                let history = DATA_CONTEXT.history.snapshot();
                let mut menu = menu.label("最近打开的项目");
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
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();
                    let current = path
                        .to_string_lossy()
                        .trim_start_matches("\\\\?\\")
                        .eq_ignore_ascii_case(
                            DATA_CONTEXT
                                .project_root()
                                .to_string_lossy()
                                .trim_start_matches("\\\\?\\"),
                        );
                    menu = menu.item(Self::toolbar_item(
                        name,
                        ToolbarAction::OpenRecentProject(path),
                        true,
                        current,
                        &view,
                    ));
                    count += 1;
                }
                if count == 0 {
                    menu = menu.label("暂无最近项目");
                }
                menu.separator().item(Self::toolbar_item(
                    "管理项目…",
                    ToolbarAction::ManageProjects,
                    true,
                    false,
                    &view,
                ))
            })
    }

    pub(super) fn branch_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let snapshot = DATA_CONTEXT.project.git();
        let available = !snapshot.branch.is_empty() && snapshot.error.is_none();
        let label = if self.git_loading {
            "加载中…".into()
        } else if available {
            snapshot.branch.clone()
        } else {
            "Git".into()
        };
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
                menu.label(if available {
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

    pub(super) fn more_menu(&self, cx: &Context<Self>) -> impl IntoElement {
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

    pub(super) fn settings_menu(&self, cx: &Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let project = self.project_panel_open;
        let automatic = DATA_CONTEXT.project.run_configuration().automatic;
        self.toolbar_button("settings-menu", cx)
            .w(px(30.))
            .tooltip("工作区设置")
            .child(icon(IconName::Settings, 18., 0xb9bdc5))
            .dropdown_menu(move |menu, _, _| {
                menu.label("工作区设置")
                    .item(Self::toolbar_item(
                        "显示 Project 面板",
                        ToolbarAction::ToggleProject,
                        true,
                        project,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "自动识别语言和工具链",
                        ToolbarAction::AutoRun,
                        true,
                        automatic,
                        &view,
                    ))
                    .separator()
                    .item(Self::toolbar_item(
                        "编辑运行配置…",
                        ToolbarAction::EditRun,
                        true,
                        false,
                        &view,
                    ))
                    .item(Self::toolbar_item(
                        "重新加载项目树",
                        ToolbarAction::RefreshProject,
                        true,
                        false,
                        &view,
                    ))
            })
    }
}
