use gpui_kit::{
    component::{h_flex, v_flex},
    *,
};

pub(crate) fn header(height: f32, background: u32, border: u32) -> Div {
    h_flex()
        .h(px(height))
        .flex_shrink_0()
        .items_center()
        .bg(super::appearance::background(background))
        .border_b_1()
        .border_color(super::appearance::border(border))
}

pub(crate) fn settings_header(title: &'static str) -> Div {
    h_flex()
        .h(px(60.))
        .flex_shrink_0()
        .px_5()
        .gap_2()
        .items_center()
        .text_size(px(15.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title)
}

pub(crate) fn card(border: u32) -> Div {
    v_flex()
        .p_4()
        .gap_3()
        .rounded_lg()
        .bg(super::appearance::background(0xffffff))
        .border_1()
        .border_color(super::appearance::border(border))
}

pub(crate) fn separator(height: f32, margin: f32, tint: u32) -> Div {
    div()
        .w(px(1.))
        .h(px(height))
        .mx(px(margin))
        .bg(super::appearance::background(tint))
}
