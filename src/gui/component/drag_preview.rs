use gpui_kit::{component::h_flex, *};

/// Shared drag preview shell; the caller provides its icon and label.
pub(crate) fn render(
    height: f32,
    padding: f32,
    border: Hsla,
    background: Hsla,
    foreground: Hsla,
) -> Div {
    h_flex()
        .h(px(height))
        .max_w(px(280.))
        .px(px(padding))
        .gap_2()
        .items_center()
        .rounded_sm()
        .border_1()
        .border_color(border)
        .bg(background)
        .shadow_md()
        .font_family(super::appearance::ui_font())
        .text_size(px(12.))
        .text_color(foreground)
}
