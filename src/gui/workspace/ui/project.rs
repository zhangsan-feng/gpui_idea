use super::*;
use crate::data_context::DATA_CONTEXT;
use crate::{domain::project::ProjectEntry, gui::workspace::TreeRow};
use gpui_kit::component::scroll::Scrollbar;
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

impl Workspace {
    pub(super) fn render_project_panel(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(color(PANEL_BG))
            .child(self.project_header(cx))
            .child(self.project_root_row(cx))
            .child(
                h_flex()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .items_stretch()
                    .child(
                        uniform_list(
                            "project-tree",
                            self.tree_rows.len(),
                            cx.processor(|this, range: Range<usize>, _, cx| {
                                range
                                    .filter_map(|index| this.tree_rows.get(index).cloned())
                                    .map(|row| this.tree_row(row, cx))
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .with_sizing_behavior(ListSizingBehavior::Auto)
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .h_full()
                        .track_scroll(&self.project_tree_scroll),
                    )
                    .child(
                        div()
                            .id("project-tree-scrollbar")
                            .relative()
                            .h_full()
                            .w(Scrollbar::width())
                            .flex_shrink_0()
                            .child(
                                Scrollbar::vertical(&self.project_tree_scroll)
                                    .viewport_from_layout(),
                            ),
                    )
                    .child(self.project_wheel_handler(cx)),
            )
    }

    fn project_wheel_handler(&self, cx: &Context<Self>) -> impl IntoElement {
        let workspace = cx.entity().downgrade();
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture && hitbox.should_handle_scroll(window) {
                        if let Some(workspace) = workspace.upgrade() {
                            workspace.update(cx, |this, cx| this.scroll_project_lines(event, cx));
                        }
                    }
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    fn project_header(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .h(px(40.))
            .flex_shrink_0()
            .px_3()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(color(BORDER))
            .bg(color(PANEL_BG))
            .text_size(px(13.))
            .font_weight(FontWeight(600.))
            .child(div().flex_1().child("Project"))
            .child(self.small_panel_button(IconName::RefreshCw, "project-refresh", cx))
            .child(self.small_panel_button(IconName::Minus, "project-collapse", cx))
    }

    fn project_root_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let expanded = self.expanded_directories.contains(Path::new(""));
        h_flex()
            .id("project-root")
            .h(px(26.))
            .flex_shrink_0()
            .px_2()
            .gap_1()
            .items_center()
            .text_size(px(12.))
            .hover(|style| style.bg(color(0xe9edf2)))
            .drag_over::<DraggedProjectEntry>(|style, drag, _, _| {
                if valid_project_drop(&drag.path, Path::new("")) {
                    style.bg(color(0xdfeaff)).border_color(color(ACCENT))
                } else {
                    style
                }
            })
            .child(icon(
                if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                },
                13.,
                MUTED,
            ))
            .child(icon(IconName::FolderOpen, 15., 0x75808e))
            .child(
                div()
                    .text_color(color(TEXT))
                    .font_weight(FontWeight(600.))
                    .child(
                        DATA_CONTEXT
                            .project_root()
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned(),
                    ),
            )
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .pl_1()
                    .text_xs()
                    .text_color(color(0x73839a))
                    .child(DATA_CONTEXT.project_root().to_string_lossy().into_owned())
                    .truncate(),
            )
            .on_click(cx.listener(|this, _, _, cx| this.toggle_directory(PathBuf::new(), cx)))
            .on_drop(cx.listener(|this, drag: &DraggedProjectEntry, window, cx| {
                cx.stop_propagation();
                this.move_project_entry(drag.path.clone(), PathBuf::new(), window, cx);
            }))
    }

    fn tree_row(&self, row: TreeRow, cx: &mut Context<Self>) -> AnyElement {
        match row {
            TreeRow::Entry(entry, depth) => self.entry_row(entry, depth, cx),
            TreeRow::Message {
                directory,
                depth,
                text,
                retry,
            } => div()
                .id(SharedString::from(format!(
                    "directory-message-{retry}-{}",
                    directory.display()
                )))
                .h(px(25.))
                .pl(px(24. + depth as f32 * 16.))
                .pr_2()
                .text_size(px(12.))
                .text_color(color(if retry { 0xb42318 } else { MUTED }))
                .truncate()
                .child(text)
                .when(retry, |row| {
                    row.cursor_pointer()
                        .hover(|style| style.bg(color(0xffe9e4)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.load_directory(directory.clone(), cx)
                        }))
                })
                .into_any_element(),
        }
    }

    fn entry_row(&self, entry: ProjectEntry, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let path = entry.relative_path.clone();
        let expanded = self.expanded_directories.contains(&path);
        let file = ProjectFile(path.clone());
        let selected =
            !entry.is_directory && !self.open_files.is_empty() && self.selected_file == file;
        let dirty = !entry.is_directory
            && DATA_CONTEXT
                .documents
                .snapshot(&file.path(DATA_CONTEXT.project_root()))
                .is_some_and(|doc| doc.dirty());
        let (symbol, tint) = if entry.is_directory {
            (
                if expanded {
                    IconName::FolderOpen
                } else {
                    IconName::FolderClosed
                },
                0xd49a38,
            )
        } else {
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("rs" | "toml" | "lock") => (IconName::FileCode, 0xe17938),
                Some("md" | "txt") => (IconName::FileText, 0x5283cc),
                _ => (IconName::File, 0x8d939c),
            }
        };
        let row = h_flex()
            .id(SharedString::from(format!("entry-{}", path.display())))
            .h(px(25.))
            .pl(px(8. + depth as f32 * 16.))
            .pr_2()
            .gap_1()
            .items_center()
            .text_size(px(12.))
            .bg(color(if selected { SELECTED } else { PANEL_BG }))
            .when(selected, |row| row.text_color(color(0x1d4f9a)))
            .hover(|style| style.bg(color(if selected { 0xd3def0 } else { 0xe9edf2 })))
            .child(div().w(px(13.)).flex_shrink_0().when(
                entry.is_directory && !entry.is_link,
                |slot| {
                    slot.child(icon(
                        if expanded {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        },
                        13.,
                        MUTED,
                    ))
                },
            ))
            .child(icon(symbol, 14., tint))
            .child(div().min_w_0().truncate().child(entry.name.clone()))
            .when(entry.is_link, |row| {
                row.child(div().text_color(color(MUTED)).child("链接"))
            })
            .when(dirty, |row| {
                row.child(div().text_color(color(ACCENT)).child("●"))
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
            row.on_click(cx.listener(move |this, _, _, cx| this.toggle_directory(path.clone(), cx)))
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
                        style.bg(color(0xdfeaff)).border_color(color(ACCENT))
                    } else {
                        style
                    }
                })
                .on_drop(
                    cx.listener(move |this, drag: &DraggedProjectEntry, window, cx| {
                        cx.stop_propagation();
                        this.move_project_entry(drag.path.clone(), target.clone(), window, cx);
                    }),
                )
                .into_any_element()
        } else {
            let drag = DraggedProjectEntry {
                path: path.clone(),
                name: entry.name.clone(),
                is_directory: false,
            };
            row.on_click(
                cx.listener(move |this, _, window, cx| this.select_file(file.clone(), window, cx)),
            )
            .on_drag(drag, |drag, _, _, cx| cx.new(|_| drag.clone()))
            .into_any_element()
        }
    }

    fn small_panel_button(
        &self,
        name: IconName,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let button = div()
            .id(id)
            .w(px(23.))
            .h(px(23.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .hover(|style| style.bg(color(0xe7e9ed)))
            .child(icon(name, 14., MUTED));
        if id == "project-collapse" {
            button.on_click(cx.listener(|this, _, _, cx| this.toggle_project_panel(cx)))
        } else if id == "project-refresh" {
            button.on_click(cx.listener(|this, _, _, cx| this.refresh_project(cx)))
        } else {
            button
        }
    }
}

fn valid_project_drop(source: &Path, target: &Path) -> bool {
    source
        .file_name()
        .is_some_and(|name| target.join(name) != source && !target.starts_with(source))
}

impl Render for DraggedProjectEntry {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(32.))
            .max_w(px(280.))
            .px_2()
            .gap_2()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(color(ACCENT))
            .bg(color(EDITOR_BG))
            .shadow_md()
            .font_family("Microsoft YaHei UI")
            .text_size(px(12.))
            .text_color(color(TEXT))
            .child(icon(
                if self.is_directory {
                    IconName::FolderClosed
                } else {
                    IconName::File
                },
                14.,
                if self.is_directory { 0xd49a38 } else { MUTED },
            ))
            .child(div().truncate().child(self.name.clone()))
    }
}
