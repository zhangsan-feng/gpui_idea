use super::*;

impl MeasuredElement {
    /// Pass the instant captured before building the child to include build time.
    pub(crate) fn new(
        child: AnyElement,
        started: Instant,
        on_measure: impl FnMut(RenderTimings, &Window, &App) + 'static,
    ) -> Self {
        Self {
            child,
            on_measure: Box::new(on_measure),
            started,
            build_us: started.elapsed().as_micros(),
            layout_started: None,
            layout_us: 0,
            prepaint_us: 0,
        }
    }
}
