use super::super::*;
use crate::gui::workspace::editor::Editor;
use std::cell::Cell;

impl Editor {
    pub(in crate::gui::workspace) fn scroll_editor_lines(
        editor: &Entity<EditorState>,
        event: &ScrollEvent,
        pending_offset: &Cell<Point<Pixels>>,
        cx: &mut App,
    ) {
        // Touchpads and horizontal gestures continue through the native handler.
        let Some(lines) = crate::gui::component::scroll::vertical_wheel_lines(event) else {
            return;
        };
        let Some(line_height) = editor.read(cx).line_height() else {
            return;
        };
        let applied_lines = lines.signum() * 5.;
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
