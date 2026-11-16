use super::*;

impl BottomSidebar {
    pub(super) fn render_file_path(&self, cx: &Context<Self>) -> Div {
        let editor = self.peer_editor(cx).read(cx);
        let has_file = !editor.state.open_files.is_empty();
        let root = DATA_CONTEXT.project_root();
        let path = editor.state.selected_file.relative_path();
        let label_path = if DATA_CONTEXT.has_project() {
            let relative = path.strip_prefix(root).unwrap_or(path);
            if relative.is_absolute() {
                relative.to_path_buf()
            } else {
                std::path::PathBuf::from(root.file_name().unwrap_or_default()).join(relative)
            }
        } else {
            path.to_path_buf()
        };
        let parent = if has_file {
            label_path.parent().unwrap_or(std::path::Path::new(""))
        } else if DATA_CONTEXT.has_project() {
            std::path::Path::new(root.file_name().unwrap_or_default())
        } else {
            std::path::Path::new("")
        };
        let folders = crate::gui::component::path::display_path(parent)
            .split(['\\', '/'])
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" › ");
        h_flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap_2()
            .child(icon(IconName::Folder, 13., ACCENT))
            .when(!folders.is_empty(), |row| {
                row.child(div().min_w_0().truncate().child(folders))
            })
            .when(has_file && !parent.as_os_str().is_empty(), |row| {
                row.child(icon(IconName::ChevronRight, 11., 0x9ca3ad))
            })
            .when(has_file, |row| {
                row.child(
                    div()
                        .min_w_0()
                        .max_w(px(260.))
                        .truncate()
                        .child(editor.state.selected_file.name()),
                )
            })
            .when(!has_file && !DATA_CONTEXT.has_project(), |row| {
                row.child("未打开文件")
            })
    }
}
