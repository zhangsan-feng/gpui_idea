use super::*;
use crate::{domain::file_tree::ProjectClipboard, gui::workspace::FileTreeMenuAction};
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};

pub(super) fn project_menu(
    menu: PopupMenu,
    view: WeakEntity<Workspace>,
    path: PathBuf,
    is_directory: bool,
    is_link: bool,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let enabled = view.upgrade().is_some_and(|view| {
        let state = view.read(cx);
        !state.closing
            && !state.close_prompt_open
            && !state.file_operation_running
            && state.moving_path.is_none()
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
        enabled && !root && !is_link,
        &view,
        &path,
        is_directory,
    ))
    .item(menu_item(
        "删除",
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
    .separator()
    .item(menu_item(
        "复制",
        IconName::Copy,
        FileTreeMenuAction::Copy,
        enabled && !root && !is_link,
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
    view: &WeakEntity<Workspace>,
    path: &Path,
    is_directory: bool,
) -> PopupMenuItem {
    let view = view.clone();
    let path = path.to_owned();
    PopupMenuItem::new(label)
        .icon(Icon::new(icon))
        .disabled(!enabled)
        .on_click(move |_, window, cx| {
            if enabled {
                let _ = view.update(cx, |this, cx| {
                    this.file_tree_menu_action(action, path.clone(), is_directory, window, cx)
                });
            }
        })
}
