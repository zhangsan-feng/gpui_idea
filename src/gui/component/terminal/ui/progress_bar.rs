use super::super::TerminalMetrics;
use crate::domain::terminal::TerminalCell;
use gpui_kit::{Bounds, Pixels, Point, Window, fill, point, px, rgb, size};

/// Rich uses heavy horizontal lines and half-cell end caps for progress bars.
/// Paint their geometry rather than shaping each symbol through font fallback.
pub(super) fn paint(
    cell: &TerminalCell,
    origin: Point<Pixels>,
    metrics: TerminalMetrics,
    window: &mut Window,
) -> bool {
    let (start, end, heavy) = match cell.text.as_str() {
        "\u{2501}" => (0., 1., true),
        "\u{2578}" => (0., 0.5, true),
        "\u{257a}" => (0.5, 1., true),
        "\u{2500}" => (0., 1., false),
        "\u{2574}" => (0., 0.5, false),
        "\u{2576}" => (0.5, 1., false),
        _ => return false,
    };
    let thickness = (metrics.font_size / if heavy { 7. } else { 14. })
        .round()
        .max(1.);
    let top = ((metrics.cell_height - thickness) / 2.).floor();
    window.paint_quad(fill(
        Bounds::new(
            point(
                origin.x + px(start * metrics.cell_width),
                origin.y + px(top),
            ),
            size(px((end - start) * metrics.cell_width), px(thickness)),
        ),
        rgb(cell.foreground),
    ));
    if cell.underline {
        window.paint_quad(fill(
            Bounds::new(
                point(origin.x, origin.y + px(metrics.cell_height - 2.)),
                size(px(metrics.cell_width), px(1.)),
            ),
            rgb(cell.foreground),
        ));
    }
    true
}
