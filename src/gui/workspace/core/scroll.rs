use super::Workspace;
use gpui_kit::component::input::{EditorState, RopeExt as _};
use gpui_kit::{App, Context, Entity, Pixels, Point, ScrollDelta, ScrollWheelEvent, px};
use std::cell::Cell;

impl Workspace {
    pub(in crate::gui::workspace) fn scroll_editor_lines(
        editor: &Entity<EditorState>,
        event: &ScrollWheelEvent,
        pending_offset: &Cell<Point<Pixels>>,
        cx: &mut App,
    ) {
        let Some(lines) = Self::vertical_wheel_lines(event) else {
            return;
        };
        let (line_height, min_y) = {
            let state = editor.read(cx);
            let Some(line_height) = state.line_height() else {
                return;
            };
            let viewport_height = state.input_bounds().size.height;
            // Match gpui-base 0.7.3's default bottom space for this unwrapped
            // plaintext editor: half the viewport, with a minimum of 3 rows.
            let bottom_space = (viewport_height / 2.).max(line_height * 3.);
            let content_height = line_height * state.text().lines_len() as f32 + bottom_space;
            (line_height, (viewport_height - content_height).min(px(0.)))
        };
        let previous = pending_offset.get();
        let mut offset = previous;
        let requested_y = offset.y + line_height * lines;
        offset.y = requested_y.clamp(min_y, px(0.));
        // set_scroll_offset is deferred. Accumulate all packets before the next
        // paint so rapid wheel input does not overwrite earlier packets.
        pending_offset.set(offset);
        if offset != previous {
            editor.update(cx, |state, cx| state.set_scroll_offset(offset, cx));
        }
        cx.stop_propagation();
        if offset != previous {
            tracing::debug!(
                input_delta = ?event.delta,
                applied_lines = lines,
                requested_y_px = f32::from(requested_y),
                min_y_px = f32::from(min_y),
                offset_y_px = f32::from(offset.y),
                moved_px = f32::from(offset.y - previous.y),
                "editor vertical mouse wheel scrolled"
            );
        }
    }

    pub(in crate::gui::workspace) fn scroll_project_lines(
        &self,
        event: &ScrollWheelEvent,
        cx: &mut Context<Self>,
    ) {
        let Some(lines) = Self::vertical_wheel_lines(event) else {
            return;
        };
        let (handle, row_height) = {
            let scroll = self.project_tree_scroll.0.borrow();
            let Some(item_size) = scroll.last_item_size else {
                return;
            };
            if self.tree_rows.is_empty() {
                return;
            }
            // GPUI stores the viewport in item_size.item, not a single row.
            let row_height = item_size.contents.height / self.tree_rows.len() as f32;
            (scroll.base_handle.clone(), row_height)
        };
        let previous = handle.offset();
        let mut offset = previous;
        offset.y =
            (offset.y + row_height * lines).clamp((-handle.max_offset().y).min(px(0.)), px(0.));
        handle.set_offset(offset);
        cx.stop_propagation();
        if offset != previous {
            cx.notify();
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

    fn vertical_wheel_lines(event: &ScrollWheelEvent) -> Option<f32> {
        let ScrollDelta::Lines(delta) = event.delta else {
            return None;
        };
        // Keep precise touchpad gestures and horizontal wheels native.
        if delta.x != 0. || delta.y == 0. || !delta.y.is_finite() {
            return None;
        }
        Some(delta.y.clamp(-Self::WHEEL_MAX_LINES, Self::WHEEL_MAX_LINES))
    }
}
