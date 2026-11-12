use super::super::*;
use crate::application::{lifecycle::run_application, markdown};

impl Editor {
    pub(in crate::gui) fn can_drop_markdown(&self, cx: &App) -> bool {
        self.markdown_mode().is_some()
            && !self
                .state
                .external_source_files
                .contains(&self.state.selected_file)
            && !self.peer_window_bar(cx).read(cx).state.closing
            && !self.peer_window_bar(cx).read(cx).state.close_prompt_open
            && !self
                .peer_file_tree(cx)
                .read(cx)
                .state
                .file_operation_running
            && self.peer_file_tree(cx).read(cx).state.moving_path.is_none()
            && self
                .state
                .editors
                .get(&self.state.selected_file)
                .is_some_and(|editor| editor.read(cx).is_editable())
    }

    pub(in crate::gui) fn insert_markdown_files(
        &mut self,
        files: Vec<PathBuf>,
        pointer: Option<gpui_kit::Point<Pixels>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_drop_markdown(cx) {
            return;
        }
        let file = self.state.selected_file.clone();
        let editor = self.state.editors[&file].clone();
        let text = editor.read(cx).text().clone();
        let cursor = editor.read(cx).cursor();
        let offset = if let Some(pointer) = pointer {
            let Some(offset) =
                crate::component::code_editor::offset_at_point(editor.read(cx), pointer)
            else {
                tracing::warn!(?pointer, "markdown drop position unavailable");
                return;
            };
            offset
        } else {
            cursor
        };
        let path = file.path(DATA_CONTEXT.project_root());
        tracing::info!(document = %path.display(), count = files.len(), ?pointer, cursor, offset, "markdown files dropped");
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(markdown::file_references(path, files)).await;
            let _ = this.update_in(cx, |this, window, cx| {
                let current = this.can_drop_markdown(cx)
                    && this.state.selected_file == file
                    && this.state.editors.get(&file) == Some(&editor)
                    && *editor.read(cx).text() == text
                    && editor.read(cx).cursor() == cursor;
                let message = match result {
                    Ok(references) if current => {
                        editor.update(cx, |state, cx| {
                            state.set_selected_range(offset..offset, cx);
                            state.insert(references, window, cx);
                            state.focus(window, cx);
                        });
                        "已插入 Markdown 文件引用".into()
                    }
                    Ok(_) => "文档或光标已变化，请重新拖入文件".into(),
                    Err(error) => {
                        tracing::warn!(%error, "markdown file drop failed");
                        format!("插入文件引用失败：{error:#}")
                    }
                };
                this.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(
                    crate::gui::main_window::bottom_sidebar::Event::Notice(Some(message)),
                ));
                cx.notify();
            });
        })
        .detach();
    }
}
