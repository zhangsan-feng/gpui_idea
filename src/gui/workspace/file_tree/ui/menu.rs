use super::*;
use crate::gui::workspace::file_tree::FileTree;
use crate::{domain::file_tree::ProjectClipboard, gui::workspace::FileTreeMenuAction};
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};

pub(super) fn project_menu(
    menu: PopupMenu,
    view: WeakEntity<FileTree>,
    path: PathBuf,
    is_directory: bool,
    is_link: bool,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let multiple = view.upgrade().is_some_and(|view| {
        let state = &view.read(cx).state;
        state.project_selection.contains(&path) && state.project_selection.len() > 1
    });
    let enabled = view.upgrade().is_some_and(|view| {
        let tree = view.read(cx);
        let shell = tree.peer_window_bar(cx);
        !shell.read(cx).state.closing
            && !shell.read(cx).state.close_prompt_open
            && !tree.state.file_operation_running
            && tree.state.moving_path.is_none()
    });
    let root = path.as_os_str().is_empty();
    tracing::debug!(path = %path.display(), root, is_directory, is_link, enabled, "project context menu target resolved");
    let clipboard = cx.read_from_clipboard().and_then(|item| {
        item.metadata()
            .and_then(|metadata| serde_json::from_str::<ProjectClipboard>(metadata).ok())
    });
    let can_paste = clipboard.is_some_and(|item| {
        item.kind == "gpui-idea-project-entry" && item.project_root == DATA_CONTEXT.project_root()
    });
    let runnable = view.upgrade().is_some_and(|view| {
        view.read(cx).state.tree_rows.iter().any(|row| {
            matches!(row, crate::gui::workspace::TreeRow::Entry(entry, _) if entry.relative_path == path && entry.runnable)
        })
    });
    let menu = if !is_directory && !is_link && runnable {
        menu.item(menu_item(
            "运行",
            IconName::Play,
            FileTreeMenuAction::Run,
            enabled && !multiple && !DATA_CONTEXT.project.run_sessions().starting,
            &view,
            &path,
            is_directory,
        ))
        .separator()
    } else {
        menu
    };
    menu.item(menu_item(
        "新建文件",
        IconName::FilePlus,
        FileTreeMenuAction::NewFile,
        enabled && !is_link,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "新建文件夹",
        IconName::Folder,
        FileTreeMenuAction::NewDirectory,
        enabled && !is_link,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "重命名",
        IconName::Pencil,
        FileTreeMenuAction::Rename,
        enabled && !root && !is_link && !multiple,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        if multiple {
            "删除选中项"
        } else {
            "删除"
        },
        IconName::Trash,
        FileTreeMenuAction::Delete,
        enabled && !root && !is_link,
        &view,
        &path,
        is_directory,
    ))
    .separator()
    .item(menu_item(
        "打开文件所在目录",
        IconName::FolderOpen,
        FileTreeMenuAction::Reveal,
        enabled && !is_link,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "复制项目路径",
        IconName::Copy,
        FileTreeMenuAction::CopyProjectPath,
        enabled,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "复制全盘路径",
        IconName::Copy,
        FileTreeMenuAction::CopyAbsolutePath,
        enabled,
        &view,
        &path,
        is_directory,
    ))
    .separator()
    .item(menu_item(
        "复制",
        IconName::Copy,
        FileTreeMenuAction::Copy,
        enabled && !root && !is_link && !multiple,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "粘贴",
        IconName::ClipboardPaste,
        FileTreeMenuAction::Paste,
        enabled && can_paste && !is_link,
        &view,
        &path,
        is_directory,
    ))
}

fn menu_item(
    label: &'static str,
    icon: IconName,
    action: FileTreeMenuAction,
    enabled: bool,
    view: &WeakEntity<FileTree>,
    path: &Path,
    is_directory: bool,
) -> PopupMenuItem {
    let path = path.to_owned();
    crate::gui::component::menu::item(label, enabled, false, view, move |this, window, cx| {
        this.file_tree_menu_action(action, path.clone(), is_directory, window, cx)
    })
    .icon(Icon::new(icon))
}
