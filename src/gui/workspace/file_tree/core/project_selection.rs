use crate::gui::workspace::DraggedProjectEntry;
use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::file_tree::FileTree;
use crate::{
    application::lifecycle::run_application,
    domain::file_tree::{FileTreeOutcome, selection_roots},
};
use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{Context, ParentElement as _, Styled as _, Window, div, px};
use std::path::{Path, PathBuf};

impl FileTree {
    pub(in crate::gui::workspace) fn select_project_entry(
        &mut self,
        path: PathBuf,
        control: bool,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        if self.state.file_operation_running || self.state.moving_path.is_some() {
            return;
        }
        if control {
            if !self.state.project_selection.remove(&path) {
                self.state.project_selection.insert(path);
            }
        } else if !self.state.project_selection.contains(&path) {
            self.state.project_selection.clear();
            self.state.project_selection.insert(path);
        }
        self.state.project_tree_focus.focus(window, cx);
        crate::event_bus::EVENT_BUS
            .refresh_gui
            .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
    }

    pub(in crate::gui::workspace) fn activate_project_entry(
        &mut self,
        path: PathBuf,
        directory: bool,
        click_count: usize,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        if self.state.file_operation_running || self.state.moving_path.is_some() {
            return;
        }
        self.state.project_selection.clear();
        self.state.project_selection.insert(path.clone());
        if directory {
            self.toggle_directory(path, cx);
        } else if click_count == 2 {
            crate::event_bus::EVENT_BUS.gui_event.send(
                self.workspace.entity_id(),
                crate::event_bus::gui_event::GuiEvent::Editor(
                    crate::gui::workspace::editor::Event::SelectFile(ProjectFile(path)),
                ),
            );
        }
        self.state.project_tree_focus.focus(window, cx);
        crate::event_bus::EVENT_BUS
            .refresh_gui
            .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
    }

    pub(in crate::gui::workspace) fn project_drag(
        &self,
        path: PathBuf,
        name: String,
        is_directory: bool,
    ) -> DraggedProjectEntry {
        let paths = if self.state.project_selection.contains(&path) {
            selection_roots(self.state.project_selection.iter().cloned().collect())
        } else {
            vec![path.clone()]
        };
        DraggedProjectEntry {
            path,
            paths,
            name,
            is_directory,
        }
    }

    pub(in crate::gui::workspace) fn delete_project_entries_dialog(
        &mut self,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        let paths = selection_roots(paths);
        if paths.is_empty()
            || paths.iter().any(|path| path.as_os_str().is_empty())
            || self.state.file_operation_running
            || self.state.moving_path.is_some()
            || self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || window.has_active_dialog(cx)
        {
            return;
        }
        let view = cx.weak_entity();
        let names = paths
            .iter()
            .take(8)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let count = paths.len();
        window.open_dialog(cx, move |dialog, _, _| {
            let paths = paths.clone();
            let view = view.clone();
            dialog
                .title(format!("删除 {count} 个文件或目录"))
                .w(px(480.))
                .child(div().child("确定删除以下选中项及目录内的全部内容？此操作不会放入回收站。"))
                .child(div().my_3().child(names.clone()))
                .when(count > 8, |dialog| {
                    dialog.child(div().child(format!("以及另外 {} 项", count - 8)))
                })
                .footer(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("delete-selection-cancel")
                                .label("取消")
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        )
                        .child(
                            Button::new("delete-selection-confirm")
                                .danger()
                                .label("删除")
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    let _ = view.update(cx, |this, cx| {
                                        this.run_project_batch(paths.clone(), None, window, cx)
                                    });
                                }),
                        ),
                )
        });
    }

    pub(in crate::gui::workspace) fn move_project_entries(
        &mut self,
        paths: Vec<PathBuf>,
        target: PathBuf,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        self.run_project_batch(paths, Some(target), window, cx);
    }

    pub(in crate::gui::workspace) fn run_project_batch(
        &mut self,
        paths: Vec<PathBuf>,
        target: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || self.state.file_operation_running
            || self.state.moving_path.is_some()
            || self.state.inline_file_edit.is_some()
        {
            return;
        }
        let paths = selection_roots(paths);
        if paths.is_empty() {
            return;
        }
        let affected = |path: &Path| paths.iter().any(|source| path.starts_with(source));
        let busy = self
            .peer_editor(cx)
            .read(cx)
            .state
            .loading_files
            .iter()
            .chain(self.peer_editor(cx).read(cx).state.saving_files.iter())
            .any(|file| affected(file.relative_path()))
            || self.state.loading_directories.iter().any(|directory| {
                affected(directory)
                    || paths
                        .iter()
                        .any(|source| source.parent() == Some(directory.as_path()))
                    || target.as_ref() == Some(directory)
            });
        if busy {
            self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(Some(
                    "相关文件正在加载或保存，请稍后再操作".into(),
                )),
            ));
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            ]);
            return;
        }
        self.state.file_operation_running = true;
        self.state.project_batch_paths = paths.clone();
        let moving = target.is_some();
        self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                "正在{} {} 项…",
                if moving { "移动" } else { "删除" },
                paths.len()
            ))),
        ));
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                crate::application::APPLICATION
                    .project
                    .operate_entries(paths, target)
                    .await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.state.file_operation_running = false;
                this.state.project_batch_paths.clear();
                match result {
                    Ok(outcome) => {
                        let completed = outcome.completed.len();
                        for item in outcome.completed {
                            match item {
                                FileTreeOutcome::Renamed {
                                    source,
                                    destination,
                                } => this.apply_project_move(&source, &destination, window, cx),
                                FileTreeOutcome::Deleted(source) => {
                                    this.remove_deleted_entries(&source, window, cx)
                                }
                                _ => {}
                            }
                        }
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(
                                match outcome.failure {
                                    Some(error) => {
                                        format!("已完成 {completed} 项，其余已停止：{error}")
                                    }
                                    None => {
                                        format!(
                                            "已{} {completed} 项",
                                            if moving { "移动" } else { "删除" }
                                        )
                                    }
                                },
                            )),
                        ));
                    }
                    Err(error) => {
                        tracing::warn!(%error, "project selection operation failed");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "操作失败：{error:#}"
                            ))),
                        ));
                    }
                }
                this.rebuild_tree();
                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                    crate::gui::workspace::window_bar::Event::ScheduleSave,
                ));
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::FileTree,
                    crate::event_bus::refresh_gui::GuiComponent::Editor,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
            });
        })
        .detach();
    }
}
