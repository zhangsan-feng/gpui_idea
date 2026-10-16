mod core;
mod ui;

use gpui_kit::{Context, IntoElement, Render, Window};

struct SettingsSection {
    label: &'static str,
    parent: Option<&'static str>,
    expandable: bool,
}

const SECTIONS: &[SettingsSection] = &[
    SettingsSection {
        label: "Rust",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "外观与行为",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "外观",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "菜单与工具栏",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "系统设置",
        parent: Some("外观与行为"),
        expandable: true,
    },
    SettingsSection {
        label: "文件颜色",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "作用域",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "通知",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "数据编辑器与查看器",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "快速列表",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "必需插件",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "受信任的位置",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "路径变量",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "演示助手",
        parent: Some("外观与行为"),
        expandable: false,
    },
    SettingsSection {
        label: "快捷键",
        parent: None,
        expandable: false,
    },
    SettingsSection {
        label: "编辑器",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "插件",
        parent: None,
        expandable: false,
    },
    SettingsSection {
        label: "版本控制",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "构建、执行、部署",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "语言与框架",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "工具",
        parent: None,
        expandable: true,
    },
    SettingsSection {
        label: "备份与同步",
        parent: None,
        expandable: false,
    },
    SettingsSection {
        label: "高级设置",
        parent: None,
        expandable: false,
    },
];

pub(super) struct WorkspaceSettings {
    selected_section: usize,
}

impl WorkspaceSettings {
    pub(super) fn new() -> Self {
        Self {
            selected_section: 2,
        }
    }
}

impl Render for WorkspaceSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::{ParentElement as _, Styled as _};
        gpui_kit::component::v_flex()
            .size_full()
            .child(crate::component::operation_title_bar::render(
                "设置", true, cx,
            ))
            .child(self.render_view(cx))
    }
}
