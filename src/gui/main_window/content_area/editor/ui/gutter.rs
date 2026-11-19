use super::*;
use gpui_kit::component::{ActiveTheme as _, input::RopeExt as _};
use gpui_kit::{ContentMask, TextAlign, TextRun, font};

const NUMBER_LEFT_PADDING: Pixels = px(8.);
// The native editor reserves 18px for the fold controls and 6px before text.
const FOLD_AND_TEXT_GAP: Pixels = px(24.);

impl Editor {
    pub(super) fn gutter_separator(&self, state: &Entity<EditorState>) -> impl IntoElement {
        let state = state.clone();
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let state = state.read(cx);
                let Some(visible) = state.visible_row_range() else {
                    return;
                };
                let Some(line_height) = state.line_height() else {
                    return;
                };
                let Some(text_origin_offset) = state.text_origin_offset() else {
                    return;
                };
                // Search/replace rows belong to the component above its text
                // viewport. Never paint the gutter over their inputs or buttons.
                let viewport = state.input_bounds().intersect(&bounds);
                if viewport.size.width <= px(0.) || viewport.size.height <= px(0.) {
                    return;
                }
                if state.search_session().open {
                    tracing::debug!(
                        container = ?bounds,
                        text_viewport = ?viewport,
                        "editor search header excluded from gutter overlay"
                    );
                }
                // Enumerate displayed rows so a large folded block does not turn
                // gutter painting into a scan over all of its hidden buffer lines.
                let map = state.display_map();
                let first = map.buffer_line_to_display_row(visible.start);
                let last = map.buffer_line_to_display_row(
                    visible
                        .end
                        .saturating_sub(1)
                        .min(state.text().lines_len().saturating_sub(1)),
                );
                let mut rows = Vec::new();
                let mut previous = None;
                for display in first..=last {
                    let row = map.display_row_to_buffer_line(display);
                    if previous == Some(row) || !visible.contains(&row) {
                        continue;
                    }
                    previous = Some(row);
                    let offset = state.text().line_start_offset(row);
                    if let Some(line_bounds) = state.range_to_bounds(&(offset..offset)) {
                        rows.push((row, line_bounds.top()));
                    }
                }
                let gutter_left = state.input_bounds().left();
                let text_left = gutter_left + text_origin_offset;
                super::selection::paint(state, &rows, viewport, text_left, line_height, window, cx);
                super::guides::paint(state, &rows, viewport, text_left, line_height, window, cx);
                // Cover the native number column to the bottom of the editor,
                // including the extra rows it lays out below the text viewport.
                // Keep the fold controls outside this layer.
                let numbers = Bounds::from_corners(
                    point(gutter_left, viewport.top()),
                    point(
                        (text_left - FOLD_AND_TEXT_GAP).min(viewport.right()),
                        bounds.bottom(),
                    ),
                );
                if numbers.size.width > NUMBER_LEFT_PADDING {
                    let active_row = state.text().offset_to_point(state.cursor()).row;
                    let active_rows = map.visible_wrap_row_count_for_buffer_line(active_row);
                    paint_line_numbers(
                        &rows,
                        (active_row, line_height * active_rows as f32),
                        numbers,
                        line_height,
                        window,
                        cx,
                    );
                }
                // A fixed boundary sits 2px before the first code column.
                // Paint it after row backgrounds so the active line cannot break it.
                window.with_content_mask(Some(ContentMask { bounds: viewport }), |window| {
                    window.paint_quad(fill(
                        Bounds::new(
                            point(text_left - px(2.), viewport.top()),
                            size(px(1.), viewport.size.height),
                        ),
                        crate::component::appearance::border(0xe9ebef),
                    ));
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }
}

fn paint_line_numbers(
    rows: &[(usize, Pixels)],
    (active_row, active_height): (usize, Pixels),
    bounds: Bounds<Pixels>,
    line_height: Pixels,
    window: &mut Window,
    cx: &mut App,
) {
    let theme = cx.theme().clone();
    let style = &theme.highlight_theme.style;
    let mut background = style
        .editor_gutter_background
        .or(style.editor_background)
        .unwrap_or_else(|| theme.input_background());
    background.a = 1.;
    let mut run = TextRun {
        len: 0,
        font: font(theme.mono_font_family.clone()),
        color: theme.muted_foreground,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    window.with_content_mask(Some(ContentMask { bounds }), |window| {
        window.paint_layer(bounds, |window| {
            window.paint_quad(fill(bounds, background));
            for &(row, top) in rows {
                let is_active = row == active_row;
                if is_active && let Some(color) = style.editor_active_line {
                    window.paint_quad(fill(
                        Bounds::new(
                            point(bounds.left(), top),
                            size(bounds.size.width, active_height),
                        ),
                        color,
                    ));
                }
                let label = (row + 1).min(9_999_999).to_string();
                run.len = label.len();
                run.color = if is_active {
                    style.editor_active_line_number.unwrap_or(theme.foreground)
                } else {
                    style.editor_line_number.unwrap_or(theme.muted_foreground)
                };
                let line = window.text_system().shape_line(
                    label.into(),
                    theme.mono_font_size,
                    std::slice::from_ref(&run),
                    None,
                );
                let origin = point(bounds.left() + NUMBER_LEFT_PADDING, top);
                let _ = line.paint(origin, line_height, TextAlign::Left, None, window, cx);
            }
        });
    });
}
