use gpui_kit::{ScrollDelta, ScrollEvent};
const VERTICAL_WHEEL_STEP_LINES: f32 = 5.;

/// Normalize discrete vertical wheels; leave touchpad and horizontal gestures native.
pub(crate) fn vertical_wheel_lines(event: &ScrollEvent) -> Option<f32> {
    let ScrollDelta::Lines(delta) = event.delta else {
        return None;
    };
    if delta.x != 0. || delta.y == 0. || !delta.y.is_finite() {
        return None;
    }
    Some(
        delta
            .y
            .clamp(-VERTICAL_WHEEL_STEP_LINES, VERTICAL_WHEEL_STEP_LINES),
    )
}

/// Shared five-line step for each discrete vertical wheel event.
pub(crate) fn vertical_wheel_step_lines(event: &ScrollEvent) -> Option<f32> {
    vertical_wheel_lines(event).map(|lines| lines.signum() * VERTICAL_WHEEL_STEP_LINES)
}
