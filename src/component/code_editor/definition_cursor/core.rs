use gpui_kit::{
    Pixels, Point,
    component::input::{EditorState, RopeExt as _},
};

fn identifier(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

// Use the editor's laid-out glyph bounds, including gutter, scrolling and
// folding. No document parsing, LSP request or text mutation is needed.
pub(super) fn over_identifier(editor: &EditorState, pointer: Point<Pixels>) -> bool {
    if !editor.input_bounds().contains(&pointer) {
        return false;
    }
    let Some(visible) = editor.visible_row_range() else {
        return false;
    };
    let text = editor.text();
    let mut low = visible.start;
    let mut high = visible.end.min(text.lines_len());
    // Hidden folded lines map to the following visible line. Taking the last
    // row at this y selects the actual visible row rather than a hidden one.
    while low < high {
        let middle = low + (high - low) / 2;
        let start = text.line_start_offset(middle);
        if editor
            .range_to_bounds(&(start..start))
            .is_some_and(|bounds| bounds.top() <= pointer.y)
        {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low == visible.start {
        return false;
    }
    let row = low - 1;
    let start = text.line_start_offset(row);
    let line = text.slice_line(row);
    let mut count = line.len_chars();
    for ch in line.chars_at(line.len()).reversed().take(2) {
        if ch == '\n' || ch == '\r' {
            count = count.saturating_sub(1);
        } else {
            break;
        }
    }
    if count == 0 {
        return false;
    }
    let mut low = 0;
    let mut high = count + 1;
    while low < high {
        let middle = low + (high - low) / 2;
        let offset = start + line.char_to_byte_idx(middle);
        if editor
            .range_to_bounds(&(offset..offset))
            .is_some_and(|bounds| bounds.left() <= pointer.x)
        {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    if low == 0 || low > count {
        return false;
    }
    let offset = start + line.char_to_byte_idx(low - 1);
    let end = start + line.char_to_byte_idx(low);
    if !text.chars_at(offset).next().is_some_and(identifier)
        || !editor
            .range_to_bounds(&(offset..end))
            .is_some_and(|bounds| bounds.contains(&pointer))
    {
        return false;
    }
    // Numeric literals are not navigation names. Bound the backwards scan so
    // a pathological long token does not make pointer movement expensive.
    text.chars_at(offset)
        .reversed()
        .take_while(|ch| identifier(*ch))
        .take(256)
        .last()
        .or_else(|| text.chars_at(offset).next())
        .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
}
