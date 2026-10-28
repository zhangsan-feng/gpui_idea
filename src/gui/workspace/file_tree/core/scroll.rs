use crate::gui::workspace::file_tree::FileTree;
use gpui_kit::{Context, ScrollWheelEvent, px};

impl FileTree {
    pub(in crate::gui::workspace) fn scroll_project_lines(
        &self,
        event: &ScrollWheelEvent,
        cx: &mut Context<FileTree>,
    ) {
        let Some(lines) = crate::gui::component::scroll::vertical_wheel_lines(event) else {
            return;
        };
        let (handle, row_height) = {
            let scroll = self.state.project_tree_scroll.0.borrow();
            let Some(item_size) = scroll.last_item_size else {
                return;
            };
            if self.state.tree_rows.is_empty() {
                return;
            }
            // GPUI stores the viewport in item_size.item, not a single row.
            let row_height = item_size.contents.height / self.state.tree_rows.len() as f32;
            (scroll.base_handle.clone(), row_height)
        };
        let previous = handle.offset();
        let mut offset = previous;
        offset.y =
            (offset.y + row_height * lines).clamp((-handle.max_offset().y).min(px(0.)), px(0.));
        handle.set_offset(offset);
        cx.stop_propagation();
        if offset != previous {
            crate::event_bus::EVENT_BUS
                .refresh_gui
                .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
        }
        if offset != previous {
            tracing::debug!(
                input_delta = ?event.delta,
                applied_lines = lines,
                row_height_px = f32::from(row_height),
                moved_px = f32::from(offset.y - previous.y),
                "project tree vertical mouse wheel scrolled"
            );
        }
    }
}
