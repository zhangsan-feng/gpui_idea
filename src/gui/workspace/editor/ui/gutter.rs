use super::*;
use gpui_kit::component::input::RopeExt as _;

impl Editor {
    pub(super) fn gutter_separator(&self, state: &Entity<EditorState>) -> impl IntoElement {
        let state = state.clone();
        canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let state = state.read(cx);
                let Some(text_bounds) = state.text_bounds() else {
                    return;
                };
                let Some(visible) = state.visible_row_range() else {
                    return;
                };
                let start = state.text().line_start_offset(visible.start);
                let Some(line) = state.range_to_bounds(&(start..start)) else {
                    return;
                };
                // Both bounds include horizontal scrolling. Their difference is
                // the laid-out gutter width, independent of font and digit count.
                let gutter_width = line.left() - text_bounds.left();
                // Leave gap_2 (8px) between the divider and the code.
                let x = state.input_bounds().left() + gutter_width - px(8.) - px(1.);
                if gutter_width > px(0.) && x >= bounds.left() && x < bounds.right() {
                    window.paint_quad(fill(
                        Bounds::new(point(x, bounds.top()), size(px(1.), bounds.size.height)),
                        crate::component::appearance::border(0xe0e3e8),
                    ));
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }
}
