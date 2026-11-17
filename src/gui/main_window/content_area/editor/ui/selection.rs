use super::*;
use gpui_kit::component::{
    ActiveTheme as _,
    input::{DisplayPoint, RopeExt as _},
};
use gpui_kit::{ContentMask, font};

/// Fill the blank margins around native selections. Leading tabs may have no
/// glyphs, so the native start position is already past their indentation.
/// Only paint outside the native highlight to avoid translucent color overlap.
pub(super) fn paint(
    state: &EditorState,
    rows: &[(usize, Pixels, usize)],
    viewport: Bounds<Pixels>,
    text_left: Pixels,
    line_height: Pixels,
    window: &mut Window,
    cx: &App,
) {
    let selection = state.selected_range();
    if selection.is_empty()
        || !window.is_window_active()
        || !state.has_selection_focus(window, cx)
        || text_left >= viewport.right()
    {
        return;
    }
    let text = state.text();
    let first = text.offset_to_point(selection.start).row;
    let last = text.offset_to_point(selection.end).row;
    if first == last {
        return;
    }
    let theme = cx.theme();
    let font_id = window
        .text_system()
        .resolve_font(&font(theme.mono_font_family.clone()));
    let space_width = window
        .text_system()
        .layout_width(font_id, theme.mono_font_size, ' ');
    let body = Bounds::from_corners(point(text_left, viewport.top()), viewport.bottom_right());
    window.with_content_mask(Some(ContentMask { bounds: body }), |window| {
        for &(row, _, _) in rows {
            if row < first || row > last {
                continue;
            }
            // Include indentation on every selected visual row, including
            // soft-wrapped continuations. Keep a partially selected first row
            // anchored to its actual selection start.
            if let Some(display_rows) = state.display_map().buffer_line_to_display_row_range(row) {
                for display_row in display_rows {
                    let position = state
                        .display_map()
                        .display_pos_to_buffer_pos(DisplayPoint::new(display_row, 0));
                    let start = text.line_start_offset(position.line) + position.col;
                    if selection.start > start || selection.end <= start {
                        continue;
                    }
                    let Some(caret) = state.range_to_bounds(&(start..start)) else {
                        continue;
                    };
                    let right = caret.left().min(viewport.right());
                    if right > text_left {
                        window.paint_quad(fill(
                            Bounds::new(
                                point(text_left, caret.top()),
                                size(right - text_left, line_height),
                            ),
                            theme.selection,
                        ));
                    }
                }
            }
            // The ending row's newline is outside the half-open selection.
            if row == last {
                continue;
            }
            let end = text.line_end_offset(row);
            let Some(caret) = state.range_to_bounds(&(end..end)) else {
                continue;
            };
            // A wrapped buffer line ends on its final visual row. Use the
            // laid-out caret rather than logical row counts for its position.
            let left = (caret.left() + space_width).max(text_left);
            if left < viewport.right() {
                window.paint_quad(fill(
                    Bounds::new(
                        point(left, caret.top()),
                        size(viewport.right() - left, line_height),
                    ),
                    theme.selection,
                ));
            }
        }
    });
}
