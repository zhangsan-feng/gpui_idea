use super::super::*;
use crate::application::{lifecycle::run_application, markdown};
use gpui_kit::component::WindowExt as _;

impl Editor {
    pub(in crate::gui) fn navigate_editor_link(
        &mut self,
        file: ProjectFile,
        offset: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if crate::component::code_editor::language_for_path(file.relative_path()) != "markdown" {
            self.go_to_definition(file, offset, window, cx);
            return;
        }
        if self.peer_window_bar(cx).read(cx).state.closing || window.has_active_dialog(cx) {
            return;
        }
        let Some(editor) = self.state.editors.get(&file) else {
            return;
        };
        let buffer = editor.read(cx).text().clone();
        let expected = buffer.clone();
        self.state.definition_request = self.state.definition_request.wrapping_add(1);
        let request = self.state.definition_request;
        crate::component::code_editor::dismiss_completions(editor, cx);
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move { markdown::web_link_at_offset(buffer, offset).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.state.definition_request != request
                    || this.state.selected_file != file
                    || this.peer_window_bar(cx).read(cx).state.closing
                    || !window.is_window_active()
                    || window.has_active_dialog(cx)
                    || this.state.editors.get(&file).is_none_or(|editor| {
                        let editor = editor.read(cx);
                        editor.cursor() != offset || editor.text() != &expected
                            || !editor.focus_handle(cx).is_focused(window)
                    })
                { return; }
                match result {
                    Ok(Some(url)) => {
                        tracing::info!(file = %file.relative_path().display(), "Markdown web link opened");
                        cx.open_url(&url);
                    }
                    Ok(None) => this.go_to_definition(file, offset, window, cx),
                    Err(error) => {
                        tracing::warn!(%error, "Markdown web link resolution failed");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(
                            crate::gui::main_window::bottom_sidebar::Event::Notice(Some(format!("网页链接打开失败：{error:#}"))),
                        ));
                    }
                }
            });
        }).detach();
    }
}
