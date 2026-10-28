use gpui_kit::{
    component::button::{Button, ButtonCustomVariant, ButtonVariants},
    *,
};

/// Shared hit area; callers supply visuals, hover colors, hints and actions.
pub(crate) fn icon_button(id: impl Into<ElementId>, width: f32, height: f32) -> Stateful<Div> {
    div()
        .id(id)
        .w(px(width))
        .h(px(height))
        .flex()
        .items_center()
        .justify_center()
}

pub(crate) fn toolbar_button(id: impl Into<ElementId>, cx: &App) -> Button {
    Button::new(id)
        .custom(
            ButtonCustomVariant::new(cx)
                .foreground(super::appearance::color(0xdfe1e5))
                .hover(super::appearance::color(super::tokens::TOOLBAR_HOVER))
                .active(super::appearance::color(0x454a54)),
        )
        .h(px(30.))
        .px(px(6.))
        .rounded(px(4.))
        .text_size(px(13.))
        .text_color(super::appearance::foreground(0xdfe1e5))
}
