use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::editor::Editor;
use crate::{application::lifecycle::run_application, domain::document::DocumentSnapshot};
use gpui_kit::component::input::{EditorState, InputEvent, Position, RopeExt as _, TabSize};
use gpui_kit::{AppContext as _, Context, Window, point, px};
use std::{sync::Arc, time::Instant};

impl Editor {
    pub(in crate::gui::workspace) fn active_document(&self) -> Option<Arc<DocumentSnapshot>> {
        if self.state.open_files.is_empty() {
            return None;
        }
        DATA_CONTEXT
            .documents
            .snapshot(&self.state.selected_file.path(DATA_CONTEXT.project_root()))
    }

    pub(in crate::gui::workspace) fn select_file(
        &mut self,
        file: ProjectFile,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
        {
            return;
        }
        if self
            .peer_file_tree(cx)
            .read(cx)
            .state
            .project_batch_paths
            .iter()
            .any(|source| file.relative_path().starts_with(source))
            || self
                .peer_file_tree(cx)
                .read(cx)
                .state
                .moving_path
                .as_ref()
                .is_some_and(|source| file.relative_path().starts_with(source))
        {
            self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(Some(
                    "文件正在移动，请稍后打开".into(),
                )),
            ));
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            ]);
            return;
        }
        let activated = Instant::now();
        if !self.state.navigating
            && self.state.navigation.get(self.state.navigation_index) != Some(&file)
        {
            if !self.state.navigation.is_empty() {
                self.state
                    .navigation
                    .truncate(self.state.navigation_index + 1);
            }
            self.state.navigation.push(file.clone());
            if self.state.navigation.len() > 100 {
                self.state.navigation.remove(0);
            }
            self.state.navigation_index = self.state.navigation.len() - 1;
        }
        if !self.state.open_files.contains(&file) {
            self.state.open_files.push(file.clone());
        }
        self.state.selected_file = file.clone();
        self.reveal_editor_tab(&file);
        self.touch_editor(&file);
        self.prune_editors();
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        if self.peer_window_bar(cx).read(cx).state.session_ready {
            self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                crate::gui::workspace::bottom_bar::Event::Notice(None),
            ));
        }
        if crate::domain::image_preview::PreviewImageFormat::for_path(file.relative_path())
            .is_some()
        {
            self.open_image_preview(file, window, cx);
            return;
        }
        if let Some(editor) = self.state.editors.get(&file) {
            editor.update(cx, |state, cx| state.focus(window, cx));
            tracing::info!(
                file = %file.relative_path().display(),
                elapsed_us = activated.elapsed().as_micros(),
                "document activated from editor cache"
            );
            self.sync_documents(window, cx);
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            ]);
            return;
        }
        if !self.state.loading_files.insert(file.clone()) {
            crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                crate::event_bus::refresh_gui::GuiComponent::Editor,
                crate::event_bus::refresh_gui::GuiComponent::FileTree,
                crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            ]);
            return;
        }
        self.state.load_errors.remove(&file);
        let started = Instant::now();

        let documents = &APPLICATION.documents;
        tracing::info!(file = %file.relative_path().display(), "document open requested");
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
        let relative = file.relative_path().to_owned();
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move {
                if relative.is_absolute() { documents.open_external(relative).await }
                else { documents.open(relative).await }
            }).await;
            let _ = this.update_in(cx, |this, window, cx| { {
                this.state.loading_files.remove(&file);
                match result {
                    Ok(opened) => {
                        let initialization = Instant::now();
                        let tab_count = opened.initial_text.bytes().filter(|byte| *byte == b'\t').count();
                        if tab_count > 0 {
                            tracing::debug!(file = %file.relative_path().display(), tab_count, tab_size = 4, "document contains hard tabs; preserving buffer offsets for display");
                        }
                        let editor = cx.new(|cx| EditorState::new(window, cx)
                            .language(crate::component::code_editor::language_for_path(file.relative_path()))
                            .soft_wrap(false)
                            .tab_size(TabSize { tab_size: 4, hard_tabs: false })
                            .default_value(opened.initial_text));
                        crate::component::code_editor::attach_completions(&editor, &opened.snapshot.path, window, cx);
                        let path = opened.snapshot.path.clone();
                        if let Some(view) = this.state.pending_file_views.remove(&path) {
                            editor.update(cx, |state, cx| {
                                let offset = state.text().position_to_offset(&Position::new(view.cursor_line, view.cursor_character));
                                state.set_selected_range(offset..offset, cx);
                                state.set_scroll_offset(point(px(view.scroll_x), px(view.scroll_y)), cx);
                            });
                        }
                        if let Some(mut position) = this.state.pending_search_positions.remove(&path) {
                            editor.update(cx, |state, cx| {
                                position.line = position.line.min(state.text().lines_len().saturating_sub(1) as u32);
                                state.set_cursor_position(position, window, cx);
                            });
                        }
                        if let Err(error) = APPLICATION.documents.attach_editor_buffer(&path, editor.read(cx).text().clone(), opened.snapshot.revision) {
                            this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("{error:#}")))));
                        }
                        let subscription = cx.subscribe_in(&editor, window, move |this, state, event: &InputEvent, _, cx| { {
                            if matches!(event, InputEvent::Change) {
                                let committed = Instant::now();
                                let buffer = state.read(cx).text().clone();
                                if DATA_CONTEXT.documents.snapshot(&path).is_some_and(|snapshot| snapshot.text == buffer) {
                                    return;
                                }
                                if let Err(error) = APPLICATION.documents.edit(&path, buffer) {
                                    this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(crate::gui::workspace::bottom_bar::Event::Notice(Some(format!("{error:#}")))));
                                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::FileTree, crate::event_bus::refresh_gui::GuiComponent::BottomBar, crate::event_bus::refresh_gui::GuiComponent::WindowBar]);
                                } else {
                                    if let Some(snapshot) = DATA_CONTEXT.documents.snapshot(&path) {
                                        let file = ProjectFile::for_path(&path);
                                        this.state.editor_revisions.insert(file, snapshot.revision);
                                    }
                                    this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(crate::gui::workspace::window_bar::Event::ScheduleSave));
                                }
                                let elapsed_us = committed.elapsed().as_micros();
                                if elapsed_us > 8_000 {
                                    tracing::warn!(file = %path.display(), elapsed_us, "document snapshot commit exceeded frame budget");
                                }
                            }
                         } });
                        this.state.editor_subscriptions.insert(file.clone(), subscription);
                        this.state.editor_revisions.insert(file.clone(), opened.snapshot.revision);
                        if this.state.selected_file == file && this.state.open_files.contains(&file) {
                            editor.update(cx, |state, cx| state.focus(window, cx));
                        }
                        this.state.editors.insert(file.clone(), editor);
                        this.sync_documents(window, cx);
                        this.prune_editors();
                        let initialize_us = initialization.elapsed().as_micros();
                        tracing::info!(file = %file.relative_path().display(), initialize_us, elapsed_ms = started.elapsed().as_millis(), "document editor ready");
                        crate::infrastructure::logging::log_memory("document_editor_ready");
                        if initialize_us > 16_000 { tracing::warn!(file = %file.relative_path().display(), initialize_us, "editor initialization exceeded frame budget"); }
                    }
                    Err(error) => {
                        tracing::error!(file = %file.relative_path().display(), error = %error, "document open failed");
                        this.state.load_errors.insert(file, format!("{error:#}"));
                    }
                }
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[crate::event_bus::refresh_gui::GuiComponent::Editor, crate::event_bus::refresh_gui::GuiComponent::FileTree, crate::event_bus::refresh_gui::GuiComponent::BottomBar, crate::event_bus::refresh_gui::GuiComponent::WindowBar]);
             } });
        }).detach();
    }
}
