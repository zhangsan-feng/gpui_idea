use super::*;
use gpui_kit::component::resizable::{h_resizable, resizable_panel};

mod search;
mod toolbar;

impl Workspace {
    pub(in crate::gui::workspace) fn render_view(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .id("workspace")
            .key_context("Workspace")
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                cx.stop_propagation();
                this.open_dropped_files(paths, window, cx);
            }))
            .drag_over::<ExternalPaths>(|style, _, _, _| {
                style.border_1().border_color(color(ACCENT))
            })
            .on_action(
                cx.listener(|this, _: &SaveDocument, window, cx| this.save_current(window, cx)),
            )
            .on_action(cx.listener(|this, _: &CloseAllDocuments, _, cx| {
                this.close_all_editor_tabs(cx);
            }))
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .font_family("Microsoft YaHei UI")
            .text_size(px(12.))
            .bg(color(EDITOR_BG))
            .text_color(color(TEXT))
            .child(self.render_toolbar(window, cx))
            .child(self.render_body(cx))
            .child(self.render_status_bar(cx))
    }

    fn render_body(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .items_stretch()
            .overflow_hidden()
            .child(self.render_activity_bar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .h_full()
                    .child(self.render_bottom_area(cx)),
            )
            .child(self.render_right_tool_bar())
    }

    pub(in crate::gui::workspace::ui) fn render_workbench(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !self.project_panel_open {
            return self.render_editor(cx).into_any_element();
        }
        h_resizable("workspace-columns")
            .with_state(&self.project_resize)
            .on_resize(cx.listener(
                |this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                    tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace columns resized");
                    this.schedule_save(cx);
                },
            ))
            .child(
                resizable_panel()
                    .size(px(DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.project_width)
                        .unwrap_or(300.)))
                    .size_range(px(180.)..px(480.))
                    .flex_none()
                    .child(self.render_project_panel(cx)),
            )
            .child(
                resizable_panel()
                    .size_range(px(360.)..Pixels::MAX)
                    .child(self.render_editor(cx)),
            )
            .into_any_element()
    }
}
