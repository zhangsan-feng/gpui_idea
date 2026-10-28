use gpui_kit::{AnyElement, App, Window};
use std::time::Instant;

mod external;
mod ui;

/// CPU timings in microseconds; excludes GPU submission and presentation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderTimings {
    pub build_us: u128,
    /// Subtree intervals may include sibling work before prepaint.
    pub layout_us: u128,
    pub prepaint_us: u128,
    pub paint_us: u128,
    pub cpu_us: u128,
}

type MeasurementCallback = Box<dyn FnMut(RenderTimings, &Window, &App)>;

/// Wraps an element and reports CPU timings after its paint phase.
pub(crate) struct MeasuredElement {
    child: AnyElement,
    on_measure: MeasurementCallback,
    started: Instant,
    build_us: u128,
    layout_started: Option<Instant>,
    layout_us: u128,
    prepaint_us: u128,
}
