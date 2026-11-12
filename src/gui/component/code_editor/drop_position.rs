use gpui_kit::{
    Pixels, Point,
    component::input::{DisplayPoint, EditorState, RopeExt as _},
};

/// Resolve against painted caret bounds so scrolling, tabs, Unicode, wrapping
/// and folded rows use the same geometry as the editor itself.
pub(super) fn offset_at_point(editor: &EditorState, pointer: Point<Pixels>) -> Option<usize> {
    let text = editor.text();
    if text.len() == 0 {
        return Some(0);
    }
    let geometry = editor.prepaint_caret_geometry()?;
    // Anchor to a visible line, since the old caret may be scrolled off screen.
    let anchor_line = editor.visible_row_range()?.start;
    let anchor = text.line_start_offset(anchor_line);
    let caret = editor.range_to_bounds(&(anchor..anchor))?;
    let display = editor.display_map();
    let anchor_row = display
        .buffer_pos_to_display_pos(gpui_kit::component::input::BufferPoint::new(anchor_line, 0))
        .row;
    let delta = ((pointer.y - caret.top()) / geometry.line_height()).floor() as isize;
    let row = anchor_row
        .saturating_add_signed(delta)
        .min(display.display_row_count().saturating_sub(1));
    let start_point = display.display_pos_to_buffer_pos(DisplayPoint::new(row, 0));
    let start = text.line_start_offset(start_point.line) + start_point.col;
    let line = text.slice_line(start_point.line);
    let line_end = text.line_start_offset(start_point.line) + line.len()
        - line
            .chars_at(line.len())
            .reversed()
            .take_while(|ch| matches!(ch, '\r' | '\n'))
            .map(char::len_utf8)
            .sum::<usize>();
    let end = if row + 1 < display.display_row_count() {
        let next = display.display_pos_to_buffer_pos(DisplayPoint::new(row + 1, 0));
        (text.line_start_offset(next.line) + next.col).min(line_end)
    } else {
        line_end
    };
    let slice = text.slice(start..end);
    let count = slice.len_chars();
    let mut low = 0;
    let mut high = count;
    while low < high {
        let middle = low + (high - low) / 2;
        let offset = start + slice.char_to_byte_idx(middle);
        let bounds = editor.range_to_bounds(&(offset..offset))?;
        if bounds.left() < pointer.x {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let right = start + slice.char_to_byte_idx(low);
    if low == 0 {
        return Some(right);
    }
    let left = start + slice.char_to_byte_idx(low - 1);
    let left_bounds = editor.range_to_bounds(&(left..left))?;
    let right_bounds = editor.range_to_bounds(&(right..right))?;
    // At a wrap boundary the end caret is painted on the following row.
    // Use the preceding glyph width to choose the end of this visual row.
    let right_x = if right_bounds.top() > left_bounds.top() {
        let previous = start + slice.char_to_byte_idx(low.saturating_sub(2));
        let previous_bounds = editor.range_to_bounds(&(previous..previous))?;
        left_bounds.left() + (left_bounds.left() - previous_bounds.left()).max(gpui_kit::px(1.))
    } else {
        right_bounds.left()
    };
    Some(if pointer.x - left_bounds.left() < right_x - pointer.x {
        left
    } else {
        right
    })
}
