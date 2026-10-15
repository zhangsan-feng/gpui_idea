use super::super::{ProjectFile, Workspace};
use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::{application::lifecycle::run_application, domain::document::DocumentSnapshot};
use gpui_kit::component::input::{EditorState, InputEvent, Position, RopeExt as _, TabSize};
use gpui_kit::{AppContext as _, Context, Window, point, px};
use std::{sync::Arc, time::Instant};

impl Workspace {
    pub(in crate::gui::workspace) fn active_document(&self) -> Option<Arc<DocumentSnapshot>> {
        if self.open_files.is_empty() {
            return None;
        }
        DATA_CONTEXT
            .documents
            .snapshot(&self.selected_file.path(DATA_CONTEXT.project_root()))
    }

    pub(in crate::gui::workspace) fn select_file(
        &mut self,
        file: ProjectFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open {
            return;
        }
        if self
            .moving_path
            .as_ref()
            .is_some_and(|source| file.relative_path().starts_with(source))
        {
            self.notice = Some("文件正在移动，请稍后打开".into());
            cx.notify();
            return;
        }
        let activated = Instant::now();
        if !self.navigating && self.navigation.get(self.navigation_index) != Some(&file) {
            if !self.navigation.is_empty() {
                self.navigation.truncate(self.navigation_index + 1);
            }
            self.navigation.push(file.clone());
            if self.navigation.len() > 100 {
                self.navigation.remove(0);
            }
            self.navigation_index = self.navigation.len() - 1;
        }
        if !self.open_files.contains(&file) {
            self.open_files.push(file.clone());
        }
        self.selected_file = file.clone();
        self.reveal_editor_tab(&file);
        self.touch_editor(&file);
        self.prune_editors();
        self.schedule_save(cx);
        if self.session_ready {
            self.notice = None;
        }
        if let Some(editor) = self.editors.get(&file) {
            editor.update(cx, |state, cx| state.focus(window, cx));
            self.remember_opened(
                crate::domain::history::OpenKind::File,
                file.path(DATA_CONTEXT.project_root()),
                cx,
            );
            tracing::info!(
                file = %file.relative_path().display(),
                elapsed_us = activated.elapsed().as_micros(),
                "document activated from editor cache"
            );
            self.sync_documents(window, cx);
            cx.notify();
            return;
        }
        if !self.loading_files.insert(file.clone()) {
            cx.notify();
            return;
        }
        self.load_errors.remove(&file);
        let started = Instant::now();

        let documents = &APPLICATION.documents;
        tracing::info!(file = %file.relative_path().display(), "document open requested");
        cx.notify();
        let relative = file.relative_path().to_owned();
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                if relative.is_absolute() { documents.open_external(relative).await }
                else { documents.open(relative).await }
            }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.loading_files.remove(&file);
                match result {
                    Ok(opened) => {
                        let initialization = Instant::now();
                        let editor = cx.new(|cx| EditorState::new(window, cx)
                            .language(crate::component::code_editor::language_for_path(file.relative_path()))
                            .soft_wrap(false)
                            .tab_size(TabSize { tab_size: 4, hard_tabs: false })
                            .default_value(opened.initial_text));
                        crate::component::code_editor::attach_completions(&editor, file.relative_path(), window, cx);
                        let path = opened.snapshot.path.clone();
                        if let Some(view) = this.pending_file_views.remove(&path) {
                            editor.update(cx, |state, cx| {
                                let offset = state.text().position_to_offset(&Position::new(view.cursor_line, view.cursor_character));
                                state.set_selected_range(offset..offset, cx);
                                state.set_scroll_offset(point(px(view.scroll_x), px(view.scroll_y)), cx);
                            });
                        }
                        if let Some(mut position) = this.pending_search_positions.remove(&path) {
                            editor.update(cx, |state, cx| {
                                position.line = position.line.min(state.text().lines_len().saturating_sub(1) as u32);
                                state.set_cursor_position(position, window, cx);
                            });
                        }
                        if let Err(error) = APPLICATION.documents.attach_editor_buffer(&path, editor.read(cx).text().clone(), opened.snapshot.revision) {
                            this.notice = Some(format!("{error:#}"));
                        }
                        let subscription = cx.subscribe_in(&editor, window, move |this, state, event: &InputEvent, _, cx| {
                            if matches!(event, InputEvent::Change) {
                                let committed = Instant::now();
                                let buffer = state.read(cx).text().clone();
                                if let Err(error) = APPLICATION.documents.edit(&path, buffer) {
                                    this.notice = Some(format!("{error:#}"));
                                    cx.notify();
                                } else {
                                    if let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&path) {
                                        let file = this.project_file(&path);
                                        this.editor_revisions.insert(file, snapshot.revision);
                                    }
                                    this.schedule_save(cx);
                                }
                                let elapsed_us = committed.elapsed().as_micros();
                                if elapsed_us > 8_000 {
                                    tracing::warn!(file = %path.display(), elapsed_us, "document snapshot commit exceeded frame budget");
                                }
                            }
                        });
                        this.editor_subscriptions.insert(file.clone(), subscription);
                        this.editor_revisions.insert(file.clone(), opened.snapshot.revision);
                        if this.selected_file == file && this.open_files.contains(&file) {
                            editor.update(cx, |state, cx| state.focus(window, cx));
                        }
                        this.editors.insert(file.clone(), editor);
                        this.sync_documents(window, cx);
                        this.prune_editors();
                        let initialize_us = initialization.elapsed().as_micros();
                        tracing::info!(file = %file.relative_path().display(), initialize_us, elapsed_ms = started.elapsed().as_millis(), "document editor ready");
                        if initialize_us > 16_000 { tracing::warn!(file = %file.relative_path().display(), initialize_us, "editor initialization exceeded frame budget"); }
                    }
                    Err(error) => {
                        tracing::error!(file = %file.relative_path().display(), error = %error, "document open failed");
                        this.load_errors.insert(file, format!("{error:#}"));
                    }
                }
                cx.notify();
            });
        }).detach();
    }
}
