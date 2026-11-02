use super::*;
use gpui_kit::component::{
    input::{Copy, EditorState, Paste},
    native_menu::NativeMenu,
};

impl Editor {
    pub(super) fn editor_context_menu(
        &self,
        state: &Entity<EditorState>,
        cx: &Context<Self>,
    ) -> impl Fn(NativeMenu, &mut Window, &mut App) -> NativeMenu + 'static {
        let state = state.clone();
        let view = cx.weak_entity();
        let file = self.state.selected_file.clone();
        move |menu, _, cx| {
            let capabilities = state.read(cx).context_menu_capabilities();
            let runnable = DATA_CONTEXT
                .project
                .file_tree
                .is_file_runnable(file.relative_path());
            let Some(view) = view.upgrade() else {
                return menu
                    .menu_with_disabled("复制", !capabilities.is_copyable(), Box::new(Copy))
                    .menu_with_disabled("粘贴", !capabilities.is_editable(), Box::new(Paste));
            };
            let editor = view.read(cx);
            let current = editor.state.selected_file == file;
            let path = file.path(DATA_CONTEXT.project_root());
            let menu = if runnable && !editor.state.external_source_files.contains(&file) {
                menu.menu_with_icon_disabled(
                    "运行",
                    IconName::Play,
                    !current || !editor.can_run_current_file(cx),
                    Box::new(super::super::RunEditorFile),
                )
                .separator()
            } else {
                menu
            };
            let menu = menu
                .menu_with_disabled("复制", !capabilities.is_copyable(), Box::new(Copy))
                .menu_with_disabled("粘贴", !capabilities.is_editable(), Box::new(Paste))
                .separator()
                .menu_with_icon_disabled(
                    "打开文件所在目录",
                    IconName::FolderOpen,
                    !current,
                    Box::new(super::super::RevealEditorFile),
                )
                .menu_with_icon_disabled(
                    "复制项目路径",
                    IconName::Copy,
                    !current
                        || !DATA_CONTEXT.has_project()
                        || !path.starts_with(DATA_CONTEXT.project_root()),
                    Box::new(super::super::CopyEditorProjectPath),
                )
                .menu_with_icon_disabled(
                    "复制全盘路径",
                    IconName::Copy,
                    !current,
                    Box::new(super::super::CopyEditorAbsolutePath),
                );
            menu
        }
    }
}
