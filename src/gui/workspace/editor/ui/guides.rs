use super::*;
use gpui_kit::component::{ActiveTheme as _, input::RopeExt as _};

/// Keep the inner indent guides; column zero belongs to the fixed gutter border.
pub(super) fn paint(
    state: &EditorState,
    rows: &[(usize, Pixels, usize)],
    viewport: Bounds<Pixels>,
    text_left: Pixels,
    line_height: Pixels,
    window: &mut Window,
    cx: &App,
) {
    if text_left >= viewport.right() {
        return;
    }
    let theme = cx.theme();
    let run = TextRun {
        len: 4,
        font: font(theme.mono_font_family.clone()),
        color: theme.border,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let indent_width = window
        .text_system()
        .shape_line("    ".into(), theme.mono_font_size, &[run], None)
        .width;
    let color = theme.border.opacity(0.85);
    let body = Bounds::from_corners(point(text_left, viewport.top()), viewport.bottom_right());
    let mut previous_indent = 0;
    window.with_content_mask(Some(ContentMask { bounds: body }), |window| {
        for &(row, top, _) in rows {
            let line = state.text().slice_line(row);
            let mut indent = 0;
            for character in line.chars() {
                match character {
                    ' ' => indent += 1,
                    '\t' => indent += 4 - indent % 4,
                    _ => break,
                }
            }
            if line
                .chars()
                .all(|character| matches!(character, '\r' | '\n'))
            {
                indent = previous_indent;
            }
            for column in (4..indent).step_by(4) {
                let x = text_left + state.scroll_offset().x + indent_width * (column as f32 / 4.);
                if x >= text_left && x < viewport.right() {
                    window.paint_quad(fill(
                        Bounds::new(point(x, top), size(px(1.), line_height)),
                        color,
                    ));
                }
            }
            previous_indent = indent;
        }
    });
}
