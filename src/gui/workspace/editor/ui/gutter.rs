use super::*;
use gpui_kit::component::{ActiveTheme as _, input::RopeExt as _};
use gpui_kit::{ContentMask, TextAlign, TextRun, font};

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
                let theme = cx.theme().clone();
                let style = &theme.highlight_theme.style;
                let font_size = theme.mono_font_size;
                let font = font(theme.mono_font_family.clone());
                let digits = (state.text().lines_len().max(1).ilog10() as usize + 1).clamp(3, 7);
                let run = TextRun {
                    len: digits,
                    font,
                    color: style.editor_line_number.unwrap_or(theme.muted_foreground),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let number_width = window
                    .text_system()
                    .shape_line("+".repeat(digits).into(), font_size, &[run.clone()], None)
                    .width;
                let left = state.input_bounds().left();
                // Keep native numbers enabled for accessibility and active-line painting.
                // Repaint the number column and the 16px reserved by our empty gutter
                // collection. Native fold controls and their hitboxes stay uncovered.
                let width = number_width + px(16.);
                let gutter = Bounds::new(
                    point(left, viewport.top()),
                    size(width, viewport.size.height),
                );
                let background = style
                    .editor_gutter_background
                    .or(style.editor_background)
                    .unwrap_or_else(|| theme.input_background());
                let active = state.text().offset_to_point(state.cursor()).row;
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
                        rows.push((
                            row,
                            line_bounds.top(),
                            map.visible_wrap_row_count_for_buffer_line(row),
                        ));
                    }
                }
                window.with_content_mask(
                    Some(ContentMask {
                        bounds: gutter.intersect(&viewport),
                    }),
                    |window| {
                        window.paint_quad(fill(gutter, background));
                        for (row, top, wrapped_rows) in rows {
                            if row == active {
                                if let Some(color) = style.editor_active_line {
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(left, top),
                                            size(width, line_height * wrapped_rows as f32),
                                        ),
                                        color,
                                    ));
                                }
                            }
                            let label = (row + 1).min(9_999_999).to_string();
                            let mut run = run.clone();
                            run.len = label.len();
                            if row == active {
                                run.color =
                                    style.editor_active_line_number.unwrap_or(theme.foreground);
                            }
                            let line = window.text_system().shape_line(
                                label.into(),
                                font_size,
                                &[run],
                                None,
                            );
                            let origin = point(left + number_width - line.width, top);
                            let _ =
                                line.paint(origin, line_height, TextAlign::Left, None, window, cx);
                        }
                        // Two 8px gaps separate numbers, divider and native fold controls.
                        window.paint_quad(fill(
                            Bounds::new(
                                point(left + number_width + px(8.), viewport.top()),
                                size(px(1.), viewport.size.height),
                            ),
                            crate::component::appearance::border(0xe0e3e8),
                        ));
                    },
                );
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }
}
