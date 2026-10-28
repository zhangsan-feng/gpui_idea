use gpui_kit::{component::h_flex, *};

pub(crate) fn field(
    label: impl Into<SharedString>,
    label_width: f32,
    input: impl IntoElement,
) -> Div {
    h_flex()
        .gap_3()
        .items_center()
        .child(div().w(px(label_width)).flex_shrink_0().child(label.into()))
        .child(div().flex_1().min_w_0().child(input))
}
