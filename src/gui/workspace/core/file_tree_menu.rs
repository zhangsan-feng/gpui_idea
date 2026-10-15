use super::super::{FileTreeMenuAction, ProjectFile, Workspace};
use crate::{
    application::lifecycle::run_application,
    data_context::DATA_CONTEXT,
    domain::file_tree::{FileTreeOperation, FileTreeOutcome, ProjectClipboard},
};
use gpui_kit::component::{
    WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::{ClipboardItem, Context, ParentElement as _, Styled as _, Window, div, px};
use std::path::{Path, PathBuf};

impl Workspace {
    pub(in crate::gui::workspace) fn file_tree_menu_action(
        &mut self,
        action: FileTreeMenuAction,
        path: PathBuf,
        is_directory: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing
            || self.close_prompt_open
            || self.file_operation_running
            || self.moving_path.is_some()
        {
            return;
        }
        let directory = if is_directory {
            path.clone()
        } else {
            path.parent().unwrap_or(Path::new("")).to_owned()
        };
        match action {
            FileTreeMenuAction::NewFile
            | FileTreeMenuAction::NewDirectory
            | FileTreeMenuAction::Rename => {
                self.begin_inline_file_edit(action, path, directory, is_directory, window, cx);
            }
            FileTreeMenuAction::Delete => self.delete_entry_dialog(path, is_directory, window, cx),
            FileTreeMenuAction::Reveal => {
                self.run_file_tree_operation(FileTreeOperation::Reveal { source: path }, window, cx)
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
                    self.notice = Some("请先在项目文件树中复制文件或目录".into());
                    cx.notify();
                }
            }
        }
    }

    fn delete_entry_dialog(
        &mut self,
        path: PathBuf,
        is_directory: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if path.as_os_str().is_empty() || window.has_active_dialog(cx) {
            return;
        }
        let view = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, _| {
            let path = path.clone();
            let view = view.clone();
            dialog
                .title(if is_directory {
                    "删除目录"
                } else {
                    "删除文件"
                })
                .w(px(440.))
                .child(div().child(format!(
                    "确定删除 {}{}？此操作不会放入回收站。",
                    path.display(),
                    if is_directory {
                        " 及其全部内容"
                    } else {
                        ""
                    }
                )))
                .footer(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("delete-entry-cancel")
                                .label("取消")
                                .on_click(|_, window, cx| window.close_dialog(cx)),
                        )
                        .child(
                            Button::new("delete-entry-confirm")
                                .danger()
                                .label("删除")
                                .on_click(move |_, window, cx| {
                                    window.close_dialog(cx);
                                    let _ = view.update(cx, |this, cx| {
                                        this.run_file_tree_operation(
                                            FileTreeOperation::Delete {
                                                source: path.clone(),
                                            },
                                            window,
                                            cx,
                                        )
                                    });
                                }),
                        ),
                )
        });
    }

    pub(in crate::gui::workspace) fn run_file_tree_operation(
        &mut self,
        operation: FileTreeOperation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.file_operation_running
            || self.closing
            || self.close_prompt_open
            || self.moving_path.is_some()
        {
            return;
        }
        if let FileTreeOperation::Rename { source, .. } | FileTreeOperation::Delete { source } =
            &operation
        {
            if self
                .loading_files
                .iter()
                .any(|file| file.relative_path().starts_with(source))
                || self
                    .saving_files
                    .iter()
                    .any(|file| file.relative_path().starts_with(source))
            {
                let message = "相关文件正在加载或保存，请稍后再操作".to_owned();
                if matches!(&operation, FileTreeOperation::Rename { .. }) {
                    self.inline_file_edit_failed(message.clone(), window, cx);
                }
                self.notice = Some(message);
                cx.notify();
                return;
            }
        }
        self.file_operation_running = true;
        let inline_operation = matches!(
            &operation,
            FileTreeOperation::Create { .. }
                | FileTreeOperation::CreateDirectory { .. }
                | FileTreeOperation::Rename { .. }
        ) && self.inline_file_edit.is_some();
        self.notice = Some("正在处理文件操作…".into());
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move { crate::application::APPLICATION.project.file_tree_operation(operation).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.file_operation_running = false;
                match result {
                    Ok(FileTreeOutcome::DirectoryCreated(path)) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.expanded_directories.insert(path);
                        this.notice = Some("文件夹已新建".into());
                    }
                    Ok(FileTreeOutcome::Created(path)) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.select_file(ProjectFile(path), window, cx);
                        this.notice = Some("文件已新建".into());
                    }
                    Ok(FileTreeOutcome::Renamed { source, destination }) => {
                        if inline_operation { this.complete_inline_file_edit(); }
                        this.apply_project_move(&source, &destination, window, cx);
                        this.notice = Some("重命名完成".into());
                    }
                    Ok(FileTreeOutcome::Deleted(source)) => {
                        this.remove_deleted_entries(&source, window, cx);
                        this.notice = Some("删除完成".into());
                    }
                    Ok(FileTreeOutcome::Revealed) => { this.notice = Some("已打开文件所在目录".into()); }
                    Ok(FileTreeOutcome::Copied(clipboard)) => {
                        cx.write_to_clipboard(ClipboardItem::new_string_with_json_metadata(DATA_CONTEXT.project_root().join(&clipboard.source).to_string_lossy().into_owned(), clipboard));
                        this.notice = Some("已复制，可在目标目录粘贴".into());
                    }
                    Ok(FileTreeOutcome::Pasted(path)) => {
                        this.expanded_directories.insert(path.parent().unwrap_or(Path::new("")).to_owned());
                        this.notice = Some(format!("已粘贴 {}", path.display()));
                    }
                    Err(error) => {
                        if inline_operation { this.inline_file_edit_failed(format!("{error:#}"), window, cx); }
                        tracing::warn!(error = %format!("{error:#}"), "project context menu operation failed");
                        this.notice = Some(format!("文件操作失败：{error:#}"));
                    }
                }
                this.rebuild_tree();
                this.schedule_save(cx);
                cx.notify();
            });
        }).detach();
    }

    fn remove_deleted_entries(
        &mut self,
        source: &Path,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selected_removed = self.selected_file.relative_path().starts_with(source);
        self.open_files
            .retain(|file| !file.relative_path().starts_with(source));
        self.editors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.editor_subscriptions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.editor_revisions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.editor_order
            .retain(|file| !file.relative_path().starts_with(source));
        self.load_errors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.pending_file_views
            .retain(|path, _| !path.starts_with(DATA_CONTEXT.project_root().join(source)));
        self.navigation
            .retain(|file| !file.relative_path().starts_with(source));
        self.navigation_index = self
            .navigation_index
            .min(self.navigation.len().saturating_sub(1));
        self.expanded_directories
            .retain(|path| !path.starts_with(source));
        if selected_removed {
            if let Some(file) = self.open_files.first().cloned() {
                self.select_file(file, window, cx);
            }
        }
    }
}
