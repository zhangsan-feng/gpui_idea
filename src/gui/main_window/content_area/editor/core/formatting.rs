use super::super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use gpui_kit::component::input::RopeExt as _;

impl Editor {
    pub(in crate::gui) fn can_format_current(&self, cx: &App) -> bool {
        let file = &self.state.selected_file;
        self.state.open_files.contains(file)
            && !self.state.external_source_files.contains(file)
            && !self.state.formatting_files.contains(file)
            && !self.state.saving_files.contains(file)
            && !self.peer_window_bar(cx).read(cx).state.closing
            && !self.peer_window_bar(cx).read(cx).state.close_prompt_open
            && crate::domain::language_server::language_for_path(file.relative_path()).is_some()
            && self
                .state
                .editors
                .get(file)
                .is_some_and(|editor| editor.read(cx).context_menu_capabilities().is_editable())
            && DATA_CONTEXT
                .documents
                .snapshot(&file.path(DATA_CONTEXT.project_root()))
                .is_some_and(|document| !document.saving)
    }

    pub(in crate::gui) fn format_current(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.can_format_current(cx) {
            return;
        }
        let file = self.state.selected_file.clone();
        let editor = self.state.editors[&file].clone();
        let path = file.path(DATA_CONTEXT.project_root());
        self.state.formatting_files.insert(file.clone());
        self.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(
            crate::gui::main_window::bottom_sidebar::Event::Notice(Some("正在格式化代码…".into())),
        ));
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move { APPLICATION.documents.format(path).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.state.formatting_files.remove(&file);
                let message = match result {
                    Ok(Some((revision, text))) => {
                        let current = this.state.editors.get(&file).is_some_and(|current| *current == editor)
                            && this.state.editor_revisions.get(&file) == Some(&revision)
                            && DATA_CONTEXT
                                .documents
                                .snapshot(&file.path(DATA_CONTEXT.project_root()))
                                .is_some_and(|doc| doc.revision == revision && !doc.saving && doc.text == *editor.read(cx).text());
                        if current {
                            crate::component::code_editor::dismiss_completions(&editor, cx);
                            editor.update(cx, |state, cx| {
                                let mut cursor = state.cursor_position();
                                let scroll = state.scroll_offset();
                                state.replace_all(text, window, cx);
                                cursor.line = cursor.line.min(state.text().lines_len().saturating_sub(1) as u32);
                                let offset = state.text().position_to_offset(&cursor);
                                state.set_selected_range(offset..offset, cx);
                                state.set_scroll_offset(scroll, cx);
                            });
                            "代码已格式化".into()
                        } else {
                            tracing::info!(file = %file.relative_path().display(), revision, "stale manual formatting discarded");
                            "文件已变化，已取消本次格式化，请重试".into()
                        }
                    }
                    Ok(None) => "代码格式无需调整".into(),
                    Err(error) => {
                        tracing::warn!(file = %file.relative_path().display(), %error, "manual formatting failed");
                        format!("格式化失败：{error:#}")
                    }
                };
                this.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(crate::gui::main_window::bottom_sidebar::Event::Notice(Some(message))));
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
