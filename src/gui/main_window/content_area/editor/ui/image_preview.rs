use super::*;
use crate::gui::main_window::content_area::editor::Editor;
use gpui_kit::component::tooltip::Tooltip;

impl Editor {
    pub(super) fn render_image_preview(&self, cx: &mut Context<Self>) -> Div {
        let preview = self.state.image_previews.get(&self.state.selected_file);
        let error = self
            .state
            .load_errors
            .get(&self.state.selected_file)
            .or_else(|| preview.and_then(|view| view.error.as_ref()))
            .cloned();
        let zoom = preview.map_or(1., |view| view.zoom);
        let ready = error.is_none() && preview.is_some_and(|view| view.image.is_some());
        let scroll = preview.map(|view| view.scroll.clone()).unwrap_or_default();
        let controls = h_flex()
            .gap_1()
            .items_center()
            .child(self.image_zoom_button(
                "image-zoom-out",
                IconName::Minus,
                "缩小（Ctrl＋滚轮）",
                ready && zoom > 0.1,
                Some(0.8),
                cx,
            ))
            .child(
                div()
                    .min_w(px(72.))
                    .text_center()
                    .child(format!("窗口 ×{zoom:.2}")),
            )
            .child(self.image_zoom_button(
                "image-zoom-in",
                IconName::Plus,
                "放大（Ctrl＋滚轮）",
                ready && zoom < 8.,
                Some(1.25),
                cx,
            ))
            .child(
                crate::gui::component::controls::icon_button("image-zoom-fit", 72., 26.)
                    .rounded_sm()
                    .child("适应窗口")
                    .when(ready, |button| {
                        button
                            .cursor_pointer()
                            .hover(|style| {
                                style.bg(crate::component::appearance::background(TAB_HOVER))
                            })
                            .on_click(
                                cx.listener(|this, _, _, cx| this.zoom_image_preview(None, cx)),
                            )
                    })
                    .when(!ready, |button| button.opacity(0.4)),
            );
        crate::gui::component::image_preview::render(
            preview.and_then(|view| view.image.clone()),
            preview.map(|view| view.bytes),
            error,
            zoom,
            &scroll,
            controls.into_any_element(),
        )
        .on_scroll(cx.listener(|this, event: &ScrollEvent, _, cx| {
            if !event.modifiers.control {
                return;
            }
            let delta = event.delta.pixel_delta(px(20.)).y;
            if delta != px(0.) {
                this.zoom_image_preview(Some(if delta > px(0.) { 1.25 } else { 0.8 }), cx);
                cx.stop_propagation();
            }
        }))
    }

    fn image_zoom_button(
        &self,
        id: &'static str,
        name: IconName,
        hint: &'static str,
        enabled: bool,
        factor: Option<f32>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        crate::gui::component::controls::icon_button(id, 26., 26.)
            .rounded_sm()
            .child(icon(name, 15., MUTED))
            .tooltip(move |window, cx| Tooltip::new(hint).build(window, cx))
            .when(enabled, |button| {
                button
                    .cursor_pointer()
                    .hover(|style| style.bg(crate::component::appearance::background(TAB_HOVER)))
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.zoom_image_preview(factor, cx)),
                    )
            })
            .when(!enabled, |button| button.opacity(0.4))
    }
}
