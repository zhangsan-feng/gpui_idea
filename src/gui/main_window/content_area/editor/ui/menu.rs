use super::*;
use gpui_kit::component::{
    input::{Copy, EditorState, Paste},
    menu::PopupMenu,
};

impl Editor {
    pub(super) fn editor_context_menu(
        &self,
        state: &Entity<EditorState>,
        cx: &Context<Self>,
    ) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static {
        let state = state.clone();
        let view = cx.weak_entity();
        let file = self.state.selected_file.clone();
        move |menu, _, cx| {
            use crate::gui::component::menu::{context_action, context_menu};
            let menu = context_menu(menu).action_context(state.focus_handle(cx));
            let capabilities = state.read(cx).context_menu_capabilities();
            let runnable = DATA_CONTEXT
                .project
                .file_tree
                .is_file_runnable(file.relative_path());
            let Some(view) = view.upgrade() else {
                return menu
                    .item(context_action(
                        "复制",
                        IconName::Copy,
                        "Ctrl+C",
                        Box::new(Copy),
                        capabilities.is_copyable(),
                    ))
                    .item(context_action(
                        "粘贴",
                        IconName::ClipboardPaste,
                        "Ctrl+V",
                        Box::new(Paste),
                        capabilities.is_editable(),
                    ));
            };
            let editor = view.read(cx);
            let current = editor.state.selected_file == file;
            let path = file.path(DATA_CONTEXT.project_root());
            let menu = if runnable && !editor.state.external_source_files.contains(&file) {
                menu.item(context_action(
                    "运行",
                    IconName::Play,
                    "",
                    Box::new(super::super::RunEditorFile),
                    current && editor.can_run_current_file(cx),
                ))
                .separator()
            } else {
                menu
            };
            menu.item(context_action(
                "复制",
                IconName::Copy,
                "Ctrl+C",
                Box::new(Copy),
                capabilities.is_copyable(),
            ))
            .item(context_action(
                "粘贴",
                IconName::ClipboardPaste,
                "Ctrl+V",
                Box::new(Paste),
                capabilities.is_editable(),
            ))
            .separator()
            .item(context_action(
                "格式化代码",
                IconName::Code,
                "Ctrl+Alt+L",
                Box::new(super::super::FormatCode),
                current && capabilities.is_editable() && editor.can_format_current(cx),
            ))
            .separator()
            .item(context_action(
                "打开文件所在目录",
                IconName::FolderOpen,
                "",
                Box::new(super::super::RevealEditorFile),
                current,
            ))
            .item(context_action(
                "复制项目路径",
                IconName::Copy,
                "",
                Box::new(super::super::CopyEditorProjectPath),
                current
                    && DATA_CONTEXT.has_project()
                    && path.starts_with(DATA_CONTEXT.project_root()),
            ))
            .item(context_action(
                "复制全盘路径",
                IconName::Copy,
                "",
                Box::new(super::super::CopyEditorAbsolutePath),
                current,
            ))
        }
    }
}
