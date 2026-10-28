use gpui_kit::*;

pub(crate) fn centered(message: impl Into<SharedString>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.))
        .text_color(super::appearance::foreground(super::tokens::MUTED))
        .child(message.into())
        .into_any_element()
}
