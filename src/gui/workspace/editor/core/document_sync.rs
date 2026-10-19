use crate::gui::workspace::editor::Editor;
use crate::{application::lifecycle::run_application, data_context::DATA_CONTEXT};
use gpui_kit::component::input::RopeExt as _;
use gpui_kit::{Context, Window};

impl Editor {
    pub(in crate::gui::workspace) fn sync_documents(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing {
            return;
        }
        self.sync_image_previews(window, cx);
        for file in self.state.editors.keys().cloned().collect::<Vec<_>>() {
            let path = file.path(DATA_CONTEXT.project_root());
            let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&path) else {
                continue;
            };
            if self.state.selected_file == file {
                if let Some(error) = &snapshot.save_error {
                    self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(error.clone())),
                    ));
                }
            }
            if snapshot.dirty()
                || snapshot.saving
                || self.state.editor_revisions.get(&file) == Some(&snapshot.revision)
                || !self.state.syncing_editors.insert(file.clone())
            {
                continue;
            }
            let revision = snapshot.revision;
            let formatted = snapshot.formatted_revision == Some(revision);
            cx.spawn_in(window, async move |this, cx| {
                // Serialize large buffers off the GUI thread.
                let result = run_application(async move {
                    Ok(tokio::task::spawn_blocking(move || snapshot.text.to_string()).await?)
                }).await;
                let _ = this.update_in(cx, |this, window, cx| { {
                    this.state.syncing_editors.remove(&file);
                    match result {
                        Ok(text) => {
                            let current = DATA_CONTEXT.documents.snapshot(&path);
                            if current.is_some_and(|doc| !doc.dirty() && !doc.saving && doc.revision == revision) {
                                if let Some(editor) = this.state.editors.get(&file) {
                                    crate::component::code_editor::dismiss_completions(editor, cx);
                                    editor.update(cx, |state, cx| {
                                        let mut cursor = state.cursor_position();
                                        let scroll = state.scroll_offset();
                                        if formatted {
                                            // Formatting is one undoable edit; the Change handler
                                            // ignores echoes already committed to the snapshot.
                                            state.replace_all(text, window, cx);
                                        } else {
                                            state.set_value(text, window, cx);
                                        }
                                        cursor.line = cursor.line.min(state.text().lines_len().saturating_sub(1) as u32);
                                        let offset = state.text().position_to_offset(&cursor);
                                        state.set_selected_range(offset..offset, cx);
                                        state.set_scroll_offset(scroll, cx);
                                    });
                                    this.state.editor_revisions.insert(file.clone(), revision);
                                    tracing::info!(path = %path.display(), revision, "editor synchronized with document snapshot");
                                }
                            }
                            // A newer event may have arrived while this buffer was prepared.
                            this.sync_documents(window, cx);
                        }
                        Err(error) => {
                            tracing::warn!(path = %path.display(), %error, "editor snapshot synchronization failed");
                            this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("文件刷新失败：{error:#}")))));
                        }
                    }
                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::FileTree, crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
                 } });
            }).detach();
        }
    }
}
