use super::{
    message::centered as image_message,
    tokens::{BORDER, MUTED},
};
use gpui_kit::{
    component::{h_flex, v_flex},
    prelude::FluentBuilder,
    *,
};
use std::sync::Arc;

/// Render an image snapshot, its loading state, or an error without performing IO.
pub(crate) fn render(
    image: Option<Arc<Image>>,
    bytes: Option<usize>,
    error: Option<String>,
) -> Div {
    v_flex()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .overflow_hidden()
        .child(
            h_flex()
                .h(px(36.))
                .flex_shrink_0()
                .px_4()
                .gap_3()
                .items_center()
                .border_b_1()
                .border_color(crate::component::appearance::border(BORDER))
                .text_size(px(12.))
                .text_color(crate::component::appearance::foreground(MUTED))
                .child("图片预览")
                .child(div().flex_1())
                .when_some(bytes, |bar, bytes| {
                    bar.child(format!("{:.1} KB", bytes as f64 / 1024.))
                })
                .child("适应窗口"),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .p_6()
                .flex()
                .items_center()
                .justify_center()
                .bg(crate::component::appearance::background(0xedf0f4))
                .map(|view| {
                    if let Some(error) = error {
                        view.child(
                            v_flex()
                                .gap_2()
                                .text_size(px(13.))
                                .text_color(crate::component::appearance::foreground(0xb42318))
                                .child("图片无法预览")
                                .child(div().text_size(px(12.)).child(error.clone())),
                        )
                    } else if let Some(image) = image {
                        view.child(
                            img(image)
                                .size_full()
                                .object_fit(ObjectFit::Contain)
                                .with_loading(|| image_message("正在解码图片…"))
                                .with_fallback(|| {
                                    image_message("无法解码此图片，文件可能已损坏或格式不受支持")
                                }),
                        )
                    } else {
                        view.child(image_message("正在读取图片…"))
                    }
                }),
        )
}
