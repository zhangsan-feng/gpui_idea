use super::super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use crate::gui::main_window::content_area::editor::Editor;
use std::{cell::Cell, time::Duration};

impl Editor {
    pub(in crate::gui::main_window::content_area::editor) fn zoom_code_font(
        &mut self,
        larger: bool,
        cx: &mut Context<Self>,
    ) {
        let bar = &self.peer_window_bar(cx).read(cx).state;
        if bar.closing || bar.close_prompt_open {
            return;
        }
        if !APPLICATION
            .settings
            .adjust_editor_font_size(if larger { 1 } else { -1 })
        {
            return;
        }
        crate::component::appearance::apply_editor_font_size(cx);
        cx.notify();
        let timer = cx.background_executor().timer(Duration::from_millis(180));
        self.state.font_size_save = Some(cx.spawn(async move |this, cx| {
            timer.await;
            if let Err(error) = run_application(APPLICATION.settings.flush_editor_font_size()).await
            {
                tracing::error!(%error, "failed to save editor font size");
                let _ = this.update(cx, |this, _| {
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomSidebar(
                        crate::gui::main_window::bottom_sidebar::Event::Notice(Some(format!(
                            "无法保存编辑器字号：{error:#}"
                        ))),
                    ));
                });
            }
        }));
    }

    pub(in crate::gui) fn scroll_editor_lines(
        editor: &Entity<EditorState>,
        event: &ScrollEvent,
        pending_offset: &Cell<Point<Pixels>>,
        cx: &mut App,
    ) {
        // Touchpads and horizontal gestures continue through the native handler.
        let Some(applied_lines) = crate::gui::component::scroll::vertical_wheel_step_lines(event)
        else {
            return;
        };
        let Some(line_height) = editor.read(cx).line_height() else {
            return;
        };
        let mut offset = pending_offset.get();
        offset.y = (offset.y + line_height * applied_lines).min(px(0.));
        // The public API defers application until layout and clamps against the
        // native scroll extent, including wrapped rows, folds and bottom space.
        // Accumulate packets until paint so a fast wheel does not lose steps.
        pending_offset.set(offset);
        editor.update(cx, |state, cx| state.set_scroll_offset(offset, cx));
        cx.stop_propagation();
        tracing::debug!(
            input_delta = ?event.delta,
            applied_lines,
            requested_y_px = f32::from(offset.y),
            "editor five-line mouse wheel scroll requested"
        );
    }
}

impl Editor {
    pub(in crate::gui) fn scroll_markdown_lines(
        preview: &Entity<gpui_kit::base::text::TextViewState>,
        event: &ScrollEvent,
        line_height: Pixels,
        cx: &mut App,
    ) {
        let Some(lines) = crate::gui::component::scroll::vertical_wheel_step_lines(event) else {
            return;
        };
        preview
            .read(cx)
            .list_state()
            .scroll_by(-line_height * lines);
        preview.update(cx, |_, cx| cx.notify());
        cx.stop_propagation();
        tracing::debug!(input_delta = ?event.delta, applied_lines = lines,
            "markdown shared mouse wheel scroll requested");
    }
}
