use super::*;
use crate::gui::workspace::editor::Editor;

impl Editor {
    pub(super) fn render_image_preview(&self) -> Div {
        let preview = self.state.image_previews.get(&self.state.selected_file);
        let error = self
            .state
            .load_errors
            .get(&self.state.selected_file)
            .or_else(|| preview.and_then(|view| view.error.as_ref()));
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
                    .border_color(color(BORDER))
                    .text_size(px(12.))
                    .text_color(color(MUTED))
                    .child("图片预览")
                    .child(div().flex_1())
                    .when_some(preview, |bar, preview| {
                        bar.child(format!("{:.1} KB", preview.bytes as f64 / 1024.))
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
                    .bg(color(0xedf0f4))
                    .map(|view| {
                        if let Some(error) = error {
                            view.child(
                                v_flex()
                                    .gap_2()
                                    .text_size(px(13.))
                                    .text_color(color(0xb42318))
                                    .child("图片无法预览")
                                    .child(div().text_size(px(12.)).child(error.clone())),
                            )
                        } else if let Some(image) =
                            preview.and_then(|preview| preview.image.clone())
                        {
                            view.child(
                                img(image)
                                    .size_full()
                                    .object_fit(ObjectFit::Contain)
                                    .with_loading(|| image_message("正在解码图片…"))
                                    .with_fallback(|| {
                                        image_message(
                                            "无法解码此图片，文件可能已损坏或格式不受支持",
                                        )
                                    }),
                            )
                        } else {
                            view.child(image_message("正在读取图片…"))
                        }
                    }),
            )
    }
}

fn image_message(message: &'static str) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.))
        .text_color(color(MUTED))
        .child(message)
        .into_any_element()
}
