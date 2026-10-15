use super::*;
use crate::{domain::project::ProjectEntry, gui::workspace::TreeRow};

impl Workspace {
    pub(super) fn tree_row(&self, row: TreeRow, cx: &mut Context<Self>) -> AnyElement {
        match row {
            TreeRow::InlineFile(depth) => self.inline_file_row(depth, cx),
            TreeRow::Entry(entry, depth) => {
                if self.inline_file_edit.as_ref().is_some_and(|edit| {
                    matches!(
                        edit.action,
                        crate::gui::workspace::FileTreeMenuAction::Rename
                    ) && edit.source == entry.relative_path
                }) {
                    self.inline_file_row(depth, cx)
                } else {
                    self.entry_row(entry, depth, cx)
                }
            }
            TreeRow::Message {
                directory,
                depth,
                text,
                retry,
            } => {
                let palette = ProjectTreePalette::from_theme(cx.theme());
                let context_directory = directory.clone();
                div()
                    .id(SharedString::from(format!(
                        "directory-message-{retry}-{}",
                        directory.display()
                    )))
                    .w_full()
                    .min_w_0()
                    .h(px(PROJECT_TREE_ROW_HEIGHT))
                    .pl(px(PROJECT_TREE_LEFT_INSET
                        + PROJECT_TREE_ICON_SIZE
                        + depth as f32 * PROJECT_TREE_INDENT))
                    .pr_2()
                    .text_size(px(12.))
                    .text_color(if retry { palette.error } else { palette.muted })
                    .truncate()
                    .child(text)
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, _, _, _| {
                            this.project_context_target =
                                Some((context_directory.clone(), true, false));
                        }),
                    )
                    .when(retry, |row| {
                        row.cursor_pointer()
                            .hover(|style| style.bg(palette.error_hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.load_directory(directory.clone(), cx)
                            }))
                    })
                    .into_any_element()
            }
        }
    }

    fn entry_row(&self, entry: ProjectEntry, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let path = entry.relative_path.clone();
        let menu_path = path.clone();
        let menu_directory = entry.is_directory;
        let menu_link = entry.is_link;
        let expanded = self.expanded_directories.contains(&path);
        let file = ProjectFile(path.clone());
        let selected =
            !entry.is_directory && !self.open_files.is_empty() && self.selected_file == file;
        let dirty = !entry.is_directory
            && DATA_CONTEXT
                .documents
                .snapshot(&file.path(DATA_CONTEXT.project_root()))
                .is_some_and(|doc| doc.dirty());
        let row = h_flex()
            .id(SharedString::from(format!("entry-{}", path.display())))
            .relative()
            .group("project-drop-row")
            .w_full()
            .min_w_0()
            .h(px(PROJECT_TREE_ROW_HEIGHT))
            .border_1()
            .border_color(rgba(0x00000000))
            .pl(px(
                PROJECT_TREE_LEFT_INSET + depth as f32 * PROJECT_TREE_INDENT
            ))
            .pr_2()
            .gap_1()
            .items_center()
            .text_size(px(12.))
            .text_color(palette.foreground)
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, _, _, _| {
                    this.project_context_target =
                        Some((menu_path.clone(), menu_directory, menu_link));
                }),
            )
            .when(selected, |row| row.bg(palette.selected))
            .hover(|style| {
                style.bg(if selected {
                    palette.selected
                } else {
                    palette.hover
                })
            })
            .child(div().w(px(13.)).flex_shrink_0().when(
                entry.is_directory && !entry.is_link,
                |slot| {
                    slot.child(project_icon(
                        if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        },
                        PROJECT_TREE_CHEVRON_SIZE,
                        palette.muted,
                    ))
                },
            ))
            .child(project_entry_icon(
                &path,
                entry.is_directory,
                expanded,
                palette,
            ))
            .child(div().min_w_0().truncate().child(entry.name.clone()))
            .when(entry.is_link, |row| {
                row.child(div().text_color(palette.muted).child("链接"))
            })
            .when(dirty, |row| {
                row.child(project_icon(
                    IconName::Circle,
                    PROJECT_TREE_STATUS_ICON_SIZE,
                    palette.accent,
                ))
            });
        if entry.is_link {
            row.on_click(cx.listener(|this, _, _, cx| {
                this.notice = Some("符号链接或目录连接不支持展开或编辑".into());
                cx.notify();
            }))
            .into_any_element()
        } else if entry.is_directory {
            let target = path.clone();
            let highlight_target = target.clone();
            row.child(project_drop_hint(&target, palette))
                .on_click(
                    cx.listener(move |this, _, _, cx| this.toggle_directory(path.clone(), cx)),
                )
                .on_drag(
                    DraggedProjectEntry {
                        path: target.clone(),
                        name: entry.name.clone(),
                        is_directory: true,
                    },
                    |drag, _, _, cx| cx.new(|_| drag.clone()),
                )
                .drag_over::<DraggedProjectEntry>(move |style, drag, _, _| {
                    if valid_project_drop(&drag.path, &highlight_target) {
                        style
                            .bg(palette.drop_target)
                            .border_color(palette.drag_border)
                    } else {
                        style.bg(palette.error_hover).border_color(palette.error)
                    }
                })
                .on_drop(
                    cx.listener(move |this, drag: &DraggedProjectEntry, window, cx| {
                        cx.stop_propagation();
                        if valid_project_drop(&drag.path, &target) {
                            this.move_project_entry(drag.path.clone(), target.clone(), window, cx);
                        } else {
                            this.notice = Some("不能移入自身、子目录或文件当前所在目录".into());
                            cx.notify();
                        }
                    }),
                )
                .into_any_element()
        } else {
            let target = path.parent().unwrap_or(Path::new("")).to_owned();
            let highlight_target = target.clone();
            let drag = DraggedProjectEntry {
                path: path.clone(),
                name: entry.name.clone(),
                is_directory: false,
            };
            row.child(project_drop_hint(&target, palette))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.select_file(file.clone(), window, cx)
                }))
                .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
                .drag_over::<DraggedProjectEntry>(move |style, drag, _, _| {
                    if valid_project_drop(&drag.path, &highlight_target) {
                        style
                            .bg(palette.drop_target)
                            .border_color(palette.drag_border)
                    } else {
                        style.bg(palette.error_hover).border_color(palette.error)
                    }
                })
                .on_drop(
                    cx.listener(move |this, drag: &DraggedProjectEntry, window, cx| {
                        cx.stop_propagation();
                        if valid_project_drop(&drag.path, &target) {
                            this.move_project_entry(drag.path.clone(), target.clone(), window, cx);
                        } else {
                            this.notice = Some("不能移入自身、子目录或文件当前所在目录".into());
                            cx.notify();
                        }
                    }),
                )
                .into_any_element()
        }
    }
}
