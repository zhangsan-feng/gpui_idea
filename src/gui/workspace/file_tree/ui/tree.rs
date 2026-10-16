use super::*;
use crate::gui::workspace::file_tree::FileTree;
use crate::{domain::project::ProjectEntry, gui::workspace::TreeRow};

impl FileTree {
    pub(super) fn tree_row(&self, row: TreeRow, cx: &mut Context<FileTree>) -> AnyElement {
        match row {
            TreeRow::InlineFile(depth) => self.inline_file_row(depth, cx),
            TreeRow::Entry(entry, depth) => {
                if self.state.inline_file_edit.as_ref().is_some_and(|edit| {
                    matches!(edit.action, FileTreeMenuAction::Rename)
                        && edit.source == entry.relative_path
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
                        cx.listener(move |this, _, _, _cx| {
                            this.state.project_context_target =
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

    fn entry_row(
        &self,
        entry: ProjectEntry,
        depth: usize,
        cx: &mut Context<FileTree>,
    ) -> AnyElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let path = entry.relative_path.clone();
        let menu_path = path.clone();
        let selection_path = path.clone();
        let menu_directory = entry.is_directory;
        let menu_link = entry.is_link;
        let expanded = self.state.expanded_directories.contains(&path);
        let file = ProjectFile(path.clone());
        let selected = self.state.project_selection.contains(&path);
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
                cx.listener(move |this, _, window, cx| {
                    if !menu_link && !this.state.project_selection.contains(&menu_path) {
                        this.state.project_selection.clear();
                        this.state.project_selection.insert(menu_path.clone());
                    }
                    this.state.project_context_target =
                        Some((menu_path.clone(), menu_directory, menu_link));
                    this.state.project_tree_focus.focus(window, cx);
                    crate::event_bus::EVENT_BUS
                        .refresh_gui
                        .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    if !menu_link {
                        this.select_project_entry(
                            selection_path.clone(),
                            event.modifiers.control,
                            window,
                            cx,
                        );
                    }
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
            })
            // Opacity zero still lays out and paints the hint's text and icon.
            // Native window movement does not need this application-drag hint.
            .when(!entry.is_link && cx.has_active_drag(), |row| {
                let target = if entry.is_directory {
                    path.as_path()
                } else {
                    path.parent().unwrap_or(Path::new(""))
                };
                row.child(project_drop_hint(target, palette))
            });
        if entry.is_link {
            row.on_click(cx.listener(|this, _, _, _cx| {
                this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                    crate::gui::workspace::bottom_bar::Event::Notice(Some(
                        "符号链接或目录连接不支持展开或编辑".into(),
                    )),
                ));
                crate::event_bus::EVENT_BUS
                    .refresh_gui
                    .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
            }))
            .into_any_element()
        } else {
            let is_directory = entry.is_directory;
            let target = if is_directory {
                path.clone()
            } else {
                path.parent().unwrap_or(Path::new("")).to_owned()
            };
            let highlight_target = target.clone();
            let drag = self.project_drag(path.clone(), entry.name.clone(), is_directory);
            row.on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                if !event.modifiers().control {
                    this.activate_project_entry(
                        path.clone(),
                        is_directory,
                        event.click_count(),
                        window,
                        cx,
                    );
                }
            }))
            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
            .drag_over::<DraggedProjectEntry>(move |style, drag, _, _| {
                if valid_project_drop(drag, &highlight_target) {
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
                    if valid_project_drop(drag, &target) {
                        this.move_project_entries(drag.paths.clone(), target.clone(), window, cx);
                    } else {
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(
                                "不能移入自身、子目录或文件当前所在目录".into(),
                            )),
                        ));
                        crate::event_bus::EVENT_BUS
                            .refresh_gui
                            .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
                    }
                }),
            )
            .into_any_element()
        }
    }
}
