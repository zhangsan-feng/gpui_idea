use super::super::{FileTreeMenuAction, InlineFileEdit, TreeRow, Workspace};
use crate::{
    application::APPLICATION, data_context::DATA_CONTEXT, domain::file_tree::FileTreeOperation,
};
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{AppContext as _, Context, ScrollStrategy, Window};
use std::path::PathBuf;

impl Workspace {
    pub(in crate::gui::workspace) fn begin_inline_file_edit(
        &mut self,
        action: FileTreeMenuAction,
        source: PathBuf,
        directory: PathBuf,
        is_directory: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let rename = matches!(action, FileTreeMenuAction::Rename);
        let is_directory =
            matches!(action, FileTreeMenuAction::NewDirectory) || (rename && is_directory);
        let initial = if rename {
            source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            String::new()
        };
        let selection_end = if rename && !is_directory {
            source
                .file_stem()
                .map(|stem| stem.to_string_lossy().len())
                .unwrap_or(initial.len())
        } else {
            initial.len()
        };
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(if is_directory {
                    "文件夹名称"
                } else {
                    "文件名"
                })
                .default_value(initial)
        });
        let subscription = cx.subscribe_in(
            &input,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.submit_inline_file_edit(window, cx),
                InputEvent::Change => {
                    if let Some(edit) = &mut this.inline_file_edit {
                        edit.error = None;
                    }
                    cx.notify();
                }
                InputEvent::Blur => {
                    if !this.file_operation_running
                        && this
                            .inline_file_edit
                            .as_ref()
                            .is_some_and(|edit| edit.error.is_none())
                    {
                        this.cancel_inline_file_edit(cx);
                    }
                }
                _ => {}
            },
        );
        let observer = cx.observe(&input, |_, _, cx| cx.notify());
        self.inline_file_edit = Some(InlineFileEdit {
            action,
            source: source.clone(),
            directory: directory.clone(),
            is_directory,
            input: input.clone(),
            error: None,
            _subscriptions: vec![subscription, observer],
        });
        if !rename {
            self.expanded_directories.insert(PathBuf::new());
            self.expanded_directories.insert(directory.clone());
            if !DATA_CONTEXT.project.snapshots().1.contains_key(&directory) {
                self.load_directory(directory, cx);
            }
        }
        self.rebuild_tree();
        if let Some(index) = self.tree_rows.iter().position(|row| match row {
            TreeRow::InlineFile(_) => !rename,
            TreeRow::Entry(entry, _) => rename && entry.relative_path == source,
            _ => false,
        }) {
            self.project_tree_scroll
                .scroll_to_item(index, ScrollStrategy::Top);
        }
        let view = cx.entity().downgrade();
        // Context menus restore focus after their click callback; focus the row after dismissal.
        window.defer(cx, move |window, cx| {
            if view.upgrade().is_some_and(|view| {
                view.read(cx)
                    .inline_file_edit
                    .as_ref()
                    .is_some_and(|edit| edit.input == input)
            }) {
                input.update(cx, |state, cx| {
                    state.focus(window, cx);
                    state.set_selected_range(0..selection_end, cx);
                });
            }
        });
        cx.notify();
    }

    pub(in crate::gui::workspace) fn cancel_inline_file_edit(&mut self, cx: &mut Context<Self>) {
        if self.file_operation_running {
            return;
        }
        self.inline_file_edit = None;
        self.rebuild_tree();
        cx.notify();
    }

    pub(in crate::gui::workspace) fn submit_inline_file_edit(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.file_operation_running {
            return;
        }
        let Some(edit) = &self.inline_file_edit else {
            return;
        };
        let name = edit.input.read(cx).value().to_string();
        let name = match APPLICATION.project.validate_file_name(&name) {
            Ok(name) => name,
            Err(error) => {
                let message = format!("{error:#}");
                self.inline_file_edit.as_mut().unwrap().error = Some(message.clone());
                self.notice = Some(message);
                cx.notify();
                return;
            }
        };
        let operation = if matches!(edit.action, FileTreeMenuAction::Rename) {
            if edit
                .source
                .file_name()
                .is_some_and(|current| current == name.as_str())
            {
                self.cancel_inline_file_edit(cx);
                return;
            }
            FileTreeOperation::Rename {
                source: edit.source.clone(),
                name,
            }
        } else if matches!(edit.action, FileTreeMenuAction::NewDirectory) {
            FileTreeOperation::CreateDirectory {
                directory: edit.directory.clone(),
                name,
            }
        } else {
            FileTreeOperation::Create {
                directory: edit.directory.clone(),
                name,
            }
        };
        self.run_file_tree_operation(operation, window, cx);
    }

    pub(in crate::gui::workspace) fn complete_inline_file_edit(&mut self) {
        self.inline_file_edit = None;
        self.rebuild_tree();
    }

    pub(in crate::gui::workspace) fn inline_file_edit_failed(
        &mut self,
        message: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(edit) = &mut self.inline_file_edit {
            edit.error = Some(message);
            let input = edit.input.clone();
            window.defer(cx, move |window, cx| {
                input.update(cx, |state, cx| state.focus(window, cx))
            });
        }
    }
}
