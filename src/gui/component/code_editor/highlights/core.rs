use super::{Highlights, Snapshot};
use crate::application::{highlights, lifecycle::run_application};
use gpui_kit::{
    Context, Focusable as _, Window,
    component::input::{RangeDecoration, RangeDecorationStyle},
};
use std::time::Duration;

impl Highlights {
    pub(super) fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = self.editor.upgrade() else {
            return;
        };
        let state = editor.read(cx);
        let snapshot = Snapshot {
            text: state.text().clone(),
            selection: state.selected_range(),
            cursor: state.cursor(),
            focused: state.focus_handle(cx).is_focused(window),
            dark: crate::data_context::DATA_CONTEXT
                .settings
                .snapshot()
                .general
                .appearance
                .theme
                == crate::domain::settings::ColorTheme::Dark,
        };
        // Decoration updates notify the editor too; ignore those notifications.
        if self.snapshot.as_ref() == Some(&snapshot) {
            return;
        }
        if let Some(cancel) = self.cancellation.take() {
            cancel.send_replace(true);
        }
        self.pending = None;
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        let focused = snapshot.focused;
        let selection = snapshot.selection.clone();
        let cursor = snapshot.cursor;
        let text = snapshot.text.clone();
        self.snapshot = Some(snapshot);
        self.decorations.clear(cx);
        if !focused {
            return;
        }
        let path = self.path.clone();
        let executor = cx.background_executor().clone();
        let (cancel, cancelled) = tokio::sync::watch::channel(false);
        self.cancellation = Some(cancel);
        self.pending = Some(cx.spawn_in(window, async move |this, cx| {
            executor
                .timer(Duration::from_millis(if selection.is_empty() {
                    250
                } else {
                    100
                }))
                .await;
            let result = run_application(async move {
                highlights::resolve(path, text, selection, cursor, cancelled).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                if generation != this.generation {
                    return;
                }
                let Some(editor) = this.editor.upgrade() else {
                    return;
                };
                let state = editor.read(cx);
                let Some(snapshot) = &this.snapshot else {
                    return;
                };
                if !state.focus_handle(cx).is_focused(window)
                    || state.text() != &snapshot.text
                    || state.selected_range() != snapshot.selection
                    || state.cursor() != snapshot.cursor
                {
                    return;
                }
                match result {
                    Ok(ranges) => {
                        let mut color: gpui_kit::Hsla =
                            gpui_kit::rgb(if snapshot.dark { 0x60a5fa } else { 0x3b82f6 }).into();
                        color.a = if snapshot.dark { 0.22 } else { 0.14 };
                        tracing::debug!(
                            count = ranges.len(),
                            selected = !snapshot.selection.is_empty(),
                            "editor occurrence highlights applied"
                        );
                        this.decorations.set(
                            ranges
                                .into_iter()
                                .map(|range| {
                                    RangeDecoration::new(range)
                                        .with_style(RangeDecorationStyle::Fill)
                                        .with_color(color)
                                })
                                .collect(),
                            cx,
                        );
                    }
                    Err(error) => {
                        tracing::debug!(%error, "editor occurrence highlight request skipped")
                    }
                }
            });
        }));
    }
}

impl Drop for Highlights {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancellation.take() {
            cancel.send_replace(true);
        }
    }
}
