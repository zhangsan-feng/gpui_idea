use super::*;

impl TerminalView {
    pub(super) fn render_scrollbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let paint_entity = entity.clone();
        div()
            .id("terminal-scrollbar")
            .w(px(10.))
            .h_full()
            .flex_shrink_0()
            .bg(rgb(0xffffff))
            .cursor_default()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    cx.stop_propagation();
                    let Some(bounds) = this.scrollbar_bounds else {
                        return;
                    };
                    if DATA_CONTEXT.terminal.snapshot(this.id).history_size == 0 {
                        return;
                    }
                    let thumb = this.scrollbar_thumb(bounds);
                    this.scrollbar_grab = Some(if thumb.contains(&event.position) {
                        f32::from(event.position.y - thumb.top())
                    } else {
                        f32::from(thumb.size.height) * 0.5
                    });
                    this.drag_scrollbar(event.position.y, cx);
                }),
            )
            .child(
                canvas(
                    move |bounds, _, cx| {
                        entity.update(cx, |this, _| this.scrollbar_bounds = Some(bounds));
                    },
                    move |bounds, _, window, cx| {
                        let view = paint_entity.read(cx);
                        if DATA_CONTEXT.terminal.snapshot(view.id).history_size > 0 {
                            window.paint_quad(fill(bounds, rgb(0xf4f4f4)));
                            window.paint_quad(fill(view.scrollbar_thumb(bounds), rgb(0xc6c8cc)));
                        }
                    },
                )
                .size_full(),
            )
    }
}
