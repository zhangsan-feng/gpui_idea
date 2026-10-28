use crate::gui::workspace::file_tree::FileTree;
use crate::gui::workspace::{FileTreeMenuAction, ProjectFile};
use crate::{
    application::lifecycle::run_application,
    data_context::DATA_CONTEXT,
    domain::file_tree::{FileTreeOperation, FileTreeOutcome, ProjectClipboard},
};
use gpui_kit::{ClipboardItem, Context, Window};
use std::path::{Path, PathBuf};

impl FileTree {
    pub(in crate::gui::workspace) fn file_tree_menu_action(
        &mut self,
        action: FileTreeMenuAction,
        path: PathBuf,
        is_directory: bool,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || self.state.file_operation_running
            || self.state.moving_path.is_some()
        {
            return;
        }
        let directory = if is_directory {
            path.clone()
        } else {
            path.parent().unwrap_or(Path::new("")).to_owned()
        };
        match action {
            FileTreeMenuAction::Run => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
                    crate::gui::workspace::bottom_panel::Event::RunFile(path),
                ));
            }
            FileTreeMenuAction::NewFile
            | FileTreeMenuAction::NewDirectory
            | FileTreeMenuAction::Rename => {
                self.begin_inline_file_edit(action, path, directory, is_directory, window, cx);
            }
            FileTreeMenuAction::Delete => {
                let paths = if self.state.project_selection.contains(&path) {
                    self.state.project_selection.iter().cloned().collect()
                } else {
                    vec![path]
                };
                self.delete_project_entries_dialog(paths, window, cx);
            }
            FileTreeMenuAction::Reveal => {
                self.run_file_tree_operation(FileTreeOperation::Reveal { source: path }, window, cx)
            }
            FileTreeMenuAction::CopyProjectPath | FileTreeMenuAction::CopyAbsolutePath => {
                let mut paths: Vec<_> = if self.state.project_selection.contains(&path) {
                    self.state.project_selection.iter().cloned().collect()
                } else {
                    vec![path]
                };
                paths.sort();
                let absolute = matches!(action, FileTreeMenuAction::CopyAbsolutePath);
                let text = paths
                    .iter()
                    .map(|path| {
                        if absolute {
                            DATA_CONTEXT
                                .project_root()
                                .join(path)
                                .to_string_lossy()
                                .into_owned()
                        } else if path.as_os_str().is_empty() {
                            ".".to_owned()
                        } else {
                            path.to_string_lossy().into_owned()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                    crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                        "已复制 {} 项{}",
                        paths.len(),
                        if absolute {
                            "全盘路径"
                        } else {
                            "项目路径"
                        }
                    ))),
                ));
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::FileTree,
                    crate::event_bus::refresh_gui::GuiComponent::Editor,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
            }
            FileTreeMenuAction::Copy => {
                self.run_file_tree_operation(FileTreeOperation::Copy { source: path }, window, cx)
            }
            FileTreeMenuAction::Paste => {
                let clipboard = cx.read_from_clipboard().and_then(|item| {
                    item.metadata().and_then(|metadata| {
                        serde_json::from_str::<ProjectClipboard>(metadata).ok()
                    })
                });
                if let Some(clipboard) = clipboard {
                    self.run_file_tree_operation(
                        FileTreeOperation::Paste {
                            clipboard,
                            directory,
                        },
                        window,
                        cx,
                    );
                } else {
                    self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(
                            "请先在项目文件树中复制文件或目录".into(),
                        )),
                    ));
                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                        crate::event_bus::refresh_gui::GuiComponent::FileTree,
                        crate::event_bus::refresh_gui::GuiComponent::Editor,
                        crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                    ]);
                }
            }
        }
    }

    pub(in crate::gui::workspace) fn run_file_tree_operation(
        &mut self,
        operation: FileTreeOperation,
        window: &mut Window,
        cx: &mut Context<FileTree>,
    ) {
        if self.state.file_operation_running
            || self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || self.state.moving_path.is_some()
        {
            return;
        }
        if let FileTreeOperation::Rename { source, .. } | FileTreeOperation::Delete { source } =
            &operation
        {
            if self
                .peer_editor(cx)
                .read(cx)
                .state
                .loading_files
                .iter()
                .any(|file| file.relative_path().starts_with(source))
                || self
                    .peer_editor(cx)
                    .read(cx)
                    .state
                    .saving_files
                    .iter()
                    .any(|file| file.relative_path().starts_with(source))
            {
                let message = "相关文件正在加载或保存，请稍后再操作".to_owned();
                if matches!(&operation, FileTreeOperation::Rename { .. }) {
                    self.inline_file_edit_failed(message.clone(), window, cx);
                }
                self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                    crate::gui::workspace::bottom_bar::Event::Notice(Some(message)),
                ));
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::FileTree,
                    crate::event_bus::refresh_gui::GuiComponent::Editor,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
                return;
            }
        }
        self.state.file_operation_running = true;
        let inline_operation = matches!(
            &operation,
            FileTreeOperation::Create { .. }
                | FileTreeOperation::CreateDirectory { .. }
                | FileTreeOperation::Rename { .. }
        ) && self.state.inline_file_edit.is_some();
        self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
            crate::gui::workspace::bottom_bar::Event::Notice(Some("正在处理文件操作…".into())),
        ));
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move { crate::application::APPLICATION.project.file_tree_operation(operation).await }).await;
            let _ = this.update_in(cx, |this, window, cx| { {
                this.state.file_operation_running = false;
                match result {
                    Ok(FileTreeOutcome::DirectoryCreated(path)) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.state.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.state.expanded_directories.insert(path);
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("文件夹已新建".into()))));
                    }
                    Ok(FileTreeOutcome::Created(path)) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.state.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.send(crate::event_bus::gui_event::GuiEvent::Editor(crate::gui::workspace::editor::Event::SelectFile(ProjectFile(path))));
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("文件已新建".into()))));
                    }
                    Ok(FileTreeOutcome::Renamed { source, destination }) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.apply_project_move(&source, &destination, window, cx);
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("重命名完成".into()))));
                    }
                    Ok(FileTreeOutcome::Deleted(source)) => {
                        this.remove_deleted_entries(&source, window, cx);
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("删除完成".into()))));
                    }
                    Ok(FileTreeOutcome::Revealed) => { this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("已打开文件所在目录".into())))); }
                    Ok(FileTreeOutcome::Copied(clipboard)) => {
                        cx.write_to_clipboard(ClipboardItem::new_string_with_json_metadata(DATA_CONTEXT.project_root().join(&clipboard.source).to_string_lossy().into_owned(), clipboard));
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some("已复制，可在目标目录粘贴".into()))));
                    }
                    Ok(FileTreeOutcome::Pasted(path)) => {
                        this.state.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("已粘贴 {}", path.display())))));
                    }
                    Err(error) => {
                        if inline_operation { this.inline_file_edit_failed(format!("{error:#}"), window, cx); }
                        tracing::warn!(error = %format!("{error:#}"), "project context menu operation failed");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("文件操作失败：{error:#}")))));
                    }
                }
                this.rebuild_tree();
                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(crate::gui::workspace::window_bar::Event::ScheduleSave));
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree, crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
             } });
        }).detach();
    }
}

impl FileTree {
    pub(in crate::gui::workspace) fn remove_deleted_entries(
        &mut self,
        source: &Path,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.state
            .project_selection
            .retain(|path| !path.starts_with(source));
        self.state
            .expanded_directories
            .retain(|path| !path.starts_with(source));
        self.send(crate::event_bus::gui_event::GuiEvent::Editor(
            crate::gui::workspace::editor::Event::RemoveFiles(source.to_owned()),
        ));
    }
}
