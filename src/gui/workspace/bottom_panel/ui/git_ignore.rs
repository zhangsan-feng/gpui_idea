use super::*;
use crate::domain::git::GitAction;
use gpui_kit::component::{Icon, menu::PopupMenu};
use std::path::PathBuf;

impl BottomPanel {
    pub(super) fn git_ignore_menu(
        &self,
        path: PathBuf,
        unavailable: bool,
        cx: &Context<Self>,
    ) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static {
        let view = cx.weak_entity();
        let root = DATA_CONTEXT.project.git().root.clone();
        move |menu, window, cx| {
            let enabled = !unavailable
                && view.upgrade().is_some_and(|view| {
                    let state = &view.read(cx).state;
                    !state.git_busy
                        && !state.git_loading
                        && !state.git_details_loading
                        && DATA_CONTEXT.project.git().root == root
                });
            let file = path.clone();
            let file_repository = root.clone();
            let menu = crate::component::menu::context_menu(menu).item(
                crate::component::menu::context_view_item(
                    "忽略文件",
                    IconName::File,
                    "",
                    enabled,
                    &view,
                    move |this, window, cx| {
                        this.git_action(
                            GitAction::Ignore {
                                repository: file_repository.clone(),
                                path: file.clone(),
                                directory: false,
                            },
                            window,
                            cx,
                        );
                    },
                ),
            );
            let directories: Vec<_> = path
                .ancestors()
                .skip(1)
                .filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.to_owned())
                .collect();
            if directories.is_empty() {
                return menu;
            }
            let view = view.clone();
            let repository = root.clone();
            menu.submenu_with_icon(
                Some(Icon::new(IconName::Folder)),
                "忽略目录",
                window,
                cx,
                move |menu, _, _| {
                    directories.iter().fold(
                        crate::component::menu::context_menu(menu),
                        |menu, directory| {
                            let directory = directory.clone();
                            let repository = repository.clone();
                            let label =
                                format!("{}/", directory.to_string_lossy().replace('\\', "/"));
                            menu.item(crate::component::menu::context_view_item(
                                label,
                                IconName::Folder,
                                "",
                                enabled,
                                &view,
                                move |this, window, cx| {
                                    this.git_action(
                                        GitAction::Ignore {
                                            repository: repository.clone(),
                                            path: directory.clone(),
                                            directory: true,
                                        },
                                        window,
                                        cx,
                                    );
                                },
                            ))
                        },
                    )
                },
            )
        }
    }
}
