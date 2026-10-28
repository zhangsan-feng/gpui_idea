use super::*;

impl TerminalView {
    pub(in crate::gui::component::terminal) fn scrollbar_thumb(
        &self,
        bounds: Bounds<Pixels>,
    ) -> Bounds<Pixels> {
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        let height = f32::from(bounds.size.height);
        let total = snapshot.lines + snapshot.history_size;
        let thumb = (height * snapshot.lines as f32 / total.max(1) as f32)
            .max(24.)
            .min(height);
        let position = if snapshot.history_size == 0 {
            0.
        } else {
            1. - snapshot.scroll_offset.min(snapshot.history_size) as f32
                / snapshot.history_size as f32
        };
        Bounds::new(
            point(
                bounds.left(),
                bounds.top() + px((height - thumb) * position),
            ),
            size(bounds.size.width, px(thumb)),
        )
    }

    pub(in crate::gui::component::terminal) fn drag_scrollbar(
        &mut self,
        y: Pixels,
        cx: &mut Context<Self>,
    ) {
        let (Some(bounds), Some(grab)) = (self.scrollbar_bounds, self.scrollbar_grab) else {
            return;
        };
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        let thumb = self.scrollbar_thumb(bounds);
        let travel = f32::from(bounds.size.height - thumb.size.height);
        if travel <= 0. {
            return;
        }
        let position = ((f32::from(y - bounds.top()) - grab) / travel).clamp(0., 1.);
        let offset = ((1. - position) * snapshot.history_size as f32).round() as usize;
        let id = self.id;
        let previous = self.operation_task.take();
        self.operation_task = Some(cx.spawn(async move |view, cx| {
            if let Some(previous) = previous {
                previous.await;
            }
            if let Err(error) =
                run_application(async move { APPLICATION.terminal.scroll_to(id, offset).await })
                    .await
            {
                tracing::warn!(?id, error = %format!("{error:#}"), "terminal scrollbar failed");
            }
            let _ = view.update(cx, |_, cx| cx.notify());
        }));
    }
}
