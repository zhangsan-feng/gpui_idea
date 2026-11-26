use super::super::*;
use crate::application::{editor::web_link_at_offset, lifecycle::run_application};
use gpui_kit::component::WindowExt as _;
use ropey::extra::esoterica::ropes_are_instances;

impl Editor {
    pub(in crate::gui) fn navigate_editor_link(
        &mut self,
        file: ProjectFile,
        offset: usize,
        supports_lsp: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing || window.has_active_dialog(cx) {
            return;
        }
        let Some(editor) = self.state.editors.get(&file) else {
            return;
        };
        let buffer = editor.read(cx).text().clone();
        let expected = buffer.clone();
        let is_markdown =
            crate::component::code_editor::language_for_path(file.relative_path()) == "markdown";
        self.state.web_link_request = self.state.web_link_request.wrapping_add(1);
        let request = self.state.web_link_request;
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(web_link_at_offset(buffer, offset, is_markdown)).await;
            let _ = this.update_in(cx, |this, window, cx| {
                if this.state.web_link_request != request
                    || this.state.selected_file != file
                    || this.peer_window_bar(cx).read(cx).state.closing
                    || !window.is_window_active()
                    || window.has_active_dialog(cx)
                {
                    return;
                }
                let Some(editor) = this.state.editors.get(&file) else {
                    return;
                };
                let state = editor.read(cx);
                if state.cursor() != offset
                    || !state.selected_range().is_empty()
                    || !state.focus_handle(cx).is_focused(window)
                    || !ropes_are_instances(state.text(), &expected)
                {
                    return;
                }
                match result {
                    Ok(Some(url)) => {
                        crate::component::code_editor::dismiss_completions(editor, cx);
                        tracing::info!(
                            file = %file.relative_path().display(),
                            "Editor web link opened"
                        );
                        cx.open_url(&url);
                    }
                    Ok(None) if supports_lsp => this.go_to_definition(file, offset, window, cx),
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(%error, "Editor web link resolution failed");
                        let message = format!("网页链接打开失败：{error:#}");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(
                            crate::gui::main_window::bottom_sidebar::Event::Notice(Some(message)),
                        ));
                    }
                }
            });
        })
        .detach();
    }
}
