use gpui_kit::{ScrollDelta, ScrollEvent};

/// Normalize discrete vertical wheels; leave touchpad and horizontal gestures native.
pub(crate) fn vertical_wheel_lines(event: &ScrollEvent) -> Option<f32> {
    let ScrollDelta::Lines(delta) = event.delta else {
        return None;
    };
    if delta.x != 0. || delta.y == 0. || !delta.y.is_finite() {
        return None;
    }
    Some(delta.y.clamp(-5., 5.))
}
