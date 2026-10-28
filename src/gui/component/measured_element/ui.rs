use super::*;
use gpui_kit::*;

impl IntoElement for MeasuredElement {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for MeasuredElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        self.layout_started = Some(Instant::now());
        (self.child.request_layout(window, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        // Includes layout requests and intervening Taffy computation.
        // Subtree intervals also include sibling work before their prepaint.
        self.layout_us = self.layout_started.map_or(0, |at| at.elapsed().as_micros());
        let started = Instant::now();
        self.child.prepaint(window, cx);
        self.prepaint_us = started.elapsed().as_micros();
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let started = Instant::now();
        self.child.paint(window, cx);
        let paint_us = started.elapsed().as_micros();
        (self.on_measure)(
            RenderTimings {
                build_us: self.build_us,
                layout_us: self.layout_us,
                prepaint_us: self.prepaint_us,
                paint_us,
                cpu_us: self.started.elapsed().as_micros(),
            },
            window,
            cx,
        );
    }
}
