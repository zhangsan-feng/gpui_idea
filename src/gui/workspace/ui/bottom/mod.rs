use super::*;
use gpui_kit::component::resizable::{resizable_panel, v_resizable};

mod git;
mod run;
mod statistics;
mod terminal;
mod tools;

impl Workspace {
    pub(in crate::gui::workspace::ui) fn render_bottom_area(
        &self,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !self.run_panel_open {
            return self.render_workbench(cx);
        }

        v_resizable("workspace-rows")
            .with_state(&self.run_resize)
            .on_resize(cx.listener(
                |this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                    tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace rows resized");
                    this.schedule_save(cx);
                },
            ))
            .child(
                resizable_panel()
                    .size_range(px(180.)..Pixels::MAX)
                    .child(self.render_workbench(cx)),
            )
            .child(
                resizable_panel()
                    .size(px(DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.run_height)
                        .unwrap_or(250.)))
                    .size_range(px(120.)..px(500.))
                    .flex_none()
                    .child(self.render_tool_panel(cx)),
            )
            .into_any_element()
    }
}
