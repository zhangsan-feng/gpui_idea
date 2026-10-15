use super::super::Workspace;
use crate::{application::lifecycle::run_application, data_context::DATA_CONTEXT};
use gpui_kit::component::input::RopeExt as _;
use gpui_kit::{Context, Window};

impl Workspace {
    pub(in crate::gui::workspace) fn sync_documents(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing {
            return;
        }
        for file in self.editors.keys().cloned().collect::<Vec<_>>() {
            let path = file.path(DATA_CONTEXT.project_root());
            let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&path) else {
                continue;
            };
            if self.selected_file == file {
                if let Some(error) = &snapshot.save_error {
                    self.notice = Some(error.clone());
                }
            }
            if snapshot.dirty()
                || snapshot.saving
                || self.editor_revisions.get(&file) == Some(&snapshot.revision)
                || !self.syncing_editors.insert(file.clone())
            {
                continue;
            }
            let revision = snapshot.revision;
            cx.spawn_in(window, async move |this, cx| {
                // Serialize large buffers off the GUI thread.
                let result = run_application(async move {
                    Ok(tokio::task::spawn_blocking(move || snapshot.text.to_string()).await?)
                }).await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.syncing_editors.remove(&file);
                    match result {
                        Ok(text) => {
                            let current = DATA_CONTEXT.documents.snapshot(&path);
                            if current.is_some_and(|doc| !doc.dirty() && !doc.saving && doc.revision == revision) {
                                if let Some(editor) = this.editors.get(&file) {
                                    editor.update(cx, |state, cx| {
                                        let mut cursor = state.cursor_position();
                                        let scroll = state.scroll_offset();
                                        // set_value is silent: external reloads must not become local edits.
                                        state.set_value(text, window, cx);
                                        cursor.line = cursor.line.min(state.text().lines_len().saturating_sub(1) as u32);
                                        let offset = state.text().position_to_offset(&cursor);
                                        state.set_selected_range(offset..offset, cx);
                                        state.set_scroll_offset(scroll, cx);
                                    });
                                    this.editor_revisions.insert(file.clone(), revision);
                                    tracing::info!(path = %path.display(), revision, "editor synchronized with document snapshot");
                                }
                            }
                            // A newer event may have arrived while this buffer was prepared.
                            this.sync_documents(window, cx);
                        }
                        Err(error) => {
                            tracing::warn!(path = %path.display(), %error, "editor snapshot synchronization failed");
                            this.notice = Some(format!("文件刷新失败：{error:#}"));
                        }
                    }
                    cx.notify();
                });
            }).detach();
        }
    }
}
