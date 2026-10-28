use super::super::*;
use crate::gui::workspace::editor::Editor;
use gpui_kit::component::input::RopeExt as _;
use std::cell::Cell;
impl Editor {
    pub(in crate::gui::workspace) fn scroll_editor_lines(
        editor: &Entity<EditorState>,
        event: &ScrollWheelEvent,
        pending_offset: &Cell<Point<Pixels>>,
        cx: &mut App,
    ) {
        let Some(lines) = crate::gui::component::scroll::vertical_wheel_lines(event) else {
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
}
