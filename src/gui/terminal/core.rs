mod scrollbar;
use super::*;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    domain::terminal::TerminalSnapshot,
};
use gpui_kit::*;
use std::ops::Range;

impl TerminalView {
    pub(crate) fn start(&mut self, cx: &mut Context<Self>) {
        self.start_session(cx);
    }
    fn start_session(&mut self, cx: &mut Context<Self>) {
        self.focus_pending = true;
        self.starting = true;
        self.error = None;
        let id = self.id;
        let previous = self.operation_task.take();
        self.operation_task = Some(cx.spawn(async move |view, cx| {
            if let Some(previous) = previous {
                previous.await;
            }
            let result =
                run_application(async move { APPLICATION.terminal.start(id, None).await }).await;
            let _ = view.update(cx, |this, cx| {
                this.starting = false;
                this.started = result.is_ok();
                this.dimensions = (0, 0);
                if let Err(error) = result {
                    tracing::error!(?id, %error, "terminal startup failed");
                    this.error = Some(format!("无法启动终端：{error:#}"));
                }
                cx.notify();
            });
        }));
    }
    pub(crate) fn stop(&mut self, cx: &mut Context<Self>) {
        self.operation(cx, |id| async move { APPLICATION.terminal.stop(id).await });
    }
    pub(crate) fn shutdown(&mut self, cx: &mut Context<Self>) -> Task<()> {
        let previous = self.operation_task.take();
        let id = self.id;
        cx.spawn(async move |_, _| {
            if let Some(previous) = previous {
                previous.await;
            }
            if let Err(error) =
                run_application(async move { APPLICATION.terminal.stop(id).await }).await
            {
                tracing::warn!(?id,%error,"terminal shutdown failed");
            }
        })
    }
    pub(crate) fn clear(&mut self, cx: &mut Context<Self>) {
        self.operation(cx, |id| async move { APPLICATION.terminal.clear(id).await });
    }
    fn operation<F: std::future::Future<Output = anyhow::Result<()>> + Send + 'static>(
        &mut self,
        cx: &mut Context<Self>,
        operation: impl FnOnce(TerminalId) -> F,
    ) {
        let id = self.id;
        let future = operation(id);
        let previous = self.operation_task.take();
        self.operation_task = Some(cx.spawn(async move |view, cx| {
            if let Some(previous) = previous {
                previous.await;
            }
            if let Err(error) = run_application(future).await {
                tracing::error!(?id, error = %format!("{error:#}"), "terminal operation failed");
                let _ = view.update(cx, |this, cx| {
                    this.error = Some(format!("{error:#}"));
                    cx.notify();
                });
            }
        }));
    }
    fn send(&self, bytes: Vec<u8>, cx: &mut Context<Self>) {
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        if snapshot.generation == 0 || snapshot.exited {
            return;
        }
        if let Err(error) = self.input.send(bytes) {
            tracing::warn!(%error,"terminal input queue closed");
        }
        cx.notify();
    }
    pub(super) fn resize(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<Self>) {
        self.bounds = Some(bounds);
        let metrics = self.metrics();
        let columns =
            ((f32::from(bounds.size.width) / metrics.cell_width).floor() as usize).clamp(2, 1000);
        let lines =
            ((f32::from(bounds.size.height) / metrics.cell_height).floor() as usize).clamp(1, 500);
        if self.started && self.dimensions != (columns, lines) {
            self.dimensions = (columns, lines);
            self.operation(cx, |id| {
                Box::pin(async move {
                    APPLICATION
                        .terminal
                        .resize(
                            id,
                            columns,
                            lines,
                            metrics.cell_width as u16,
                            metrics.cell_height as u16,
                        )
                        .await
                })
            });
        }
    }
    pub(super) fn scroll(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        if DATA_CONTEXT.terminal.snapshot(self.id).generation == 0 {
            cx.stop_propagation();
            return;
        }
        let height = self.metrics().cell_height;
        let amount = event.delta.pixel_delta(px(height)).y;
        let lines = (f32::from(amount) / height).round() as i32;
        self.operation(cx, |id| {
            Box::pin(async move { APPLICATION.terminal.scroll(id, lines.clamp(-12, 12)).await })
        });
        cx.stop_propagation();
    }
    pub(crate) fn scroll_to_bottom(&mut self, cx: &mut Context<Self>) {
        self.operation(cx, |id| async move {
            APPLICATION.terminal.scroll_to(id, 0).await
        });
    }
    pub(super) fn cell_at(&self, position: Point<Pixels>) -> (usize, usize) {
        let Some(bounds) = self.bounds else {
            return (0, 0);
        };
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        let metrics = self.metrics();
        (
            ((f32::from(position.y - bounds.top()) / metrics.cell_height)
                .floor()
                .max(0.) as usize)
                .min(snapshot.lines.saturating_sub(1)),
            ((f32::from(position.x - bounds.left()) / metrics.cell_width)
                .floor()
                .max(0.) as usize)
                .min(snapshot.columns.saturating_sub(1)),
        )
    }
    pub(super) fn copy_text(&self, snapshot: &TerminalSnapshot) -> String {
        let (start, end) = self
            .selection
            .map(|(a, b)| (a.min(b), a.max(b)))
            .unwrap_or((
                (0, 0),
                (
                    snapshot.lines.saturating_sub(1),
                    snapshot.columns.saturating_sub(1),
                ),
            ));
        (start.0..=end.0)
            .map(|row| {
                let from = if row == start.0 { start.1 } else { 0 };
                let to = if row == end.0 {
                    end.1 + 1
                } else {
                    snapshot.columns
                };
                (from..to)
                    .filter_map(|col| snapshot.cells.get(row * snapshot.columns + col))
                    .map(|cell| cell.text.as_str())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub(crate) fn copy(&self, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(
            self.copy_text(&DATA_CONTEXT.terminal.snapshot(self.id)),
        ));
    }
    pub(super) fn paste_clipboard(&self, cx: &mut Context<Self>) {
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let text = text.replace("\r\n", "\n");
            let text = if snapshot.bracketed_paste {
                format!("\x1b[200~{text}\x1b[201~")
            } else {
                text.replace('\n', "\r")
            };
            self.send(text.into_bytes(), cx);
        }
    }
    pub(super) fn send_tab(&self, cx: &mut Context<Self>) {
        self.send(vec![b'\t'], cx);
    }
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = &event.keystroke;
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        if key.modifiers.control && key.modifiers.shift && key.key == "c" {
            cx.write_to_clipboard(ClipboardItem::new_string(self.copy_text(&snapshot)));
            cx.stop_propagation();
            return;
        }
        if key.modifiers.control && key.modifiers.shift && key.key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                let text = if snapshot.bracketed_paste {
                    format!("\x1b[200~{text}\x1b[201~")
                } else {
                    text.replace('\n', "\r")
                };
                self.send(text.into_bytes(), cx);
            }
            cx.stop_propagation();
            return;
        }
        if !self.composition.is_empty() {
            return;
        }
        let prefix = if snapshot.application_cursor {
            "\x1bO"
        } else {
            "\x1b["
        };
        let bytes = match key.key.as_str() {
            "f1" => Some("\x1bOP".into()),
            "f2" => Some("\x1bOQ".into()),
            "f3" => Some("\x1bOR".into()),
            "f4" => Some("\x1bOS".into()),
            "f5" => Some("\x1b[15~".into()),
            "f6" => Some("\x1b[17~".into()),
            "f7" => Some("\x1b[18~".into()),
            "f8" => Some("\x1b[19~".into()),
            "f9" => Some("\x1b[20~".into()),
            "f10" => Some("\x1b[21~".into()),
            "f11" => Some("\x1b[23~".into()),
            "f12" => Some("\x1b[24~".into()),
            "enter" => Some("\r".into()),
            "backspace" => Some("\x7f".into()),
            "tab" if key.modifiers.shift => Some("\x1b[Z".into()),
            "tab" => Some("\t".into()),
            "escape" => Some("\x1b".into()),
            "up" => Some(format!("{prefix}A")),
            "down" => Some(format!("{prefix}B")),
            "right" => Some(format!("{prefix}C")),
            "left" => Some(format!("{prefix}D")),
            "home" => Some(format!("{prefix}H")),
            "end" => Some(format!("{prefix}F")),
            "delete" => Some("\x1b[3~".into()),
            "insert" => Some("\x1b[2~".into()),
            "pageup" => Some("\x1b[5~".into()),
            "pagedown" => Some("\x1b[6~".into()),
            _ if key.modifiers.control && key.key.len() == 1 => {
                let byte = key.key.as_bytes()[0].to_ascii_uppercase();
                (b'@'..=b'_')
                    .contains(&byte)
                    .then(|| ((byte & 0x1f) as char).to_string())
            }
            _ if key.modifiers.alt => key.key_char.as_ref().map(|text| format!("\x1b{text}")),
            _ => None,
        };
        if let Some(bytes) = bytes {
            self.selection = None;
            self.send(bytes.into_bytes(), cx);
            cx.stop_propagation();
            window.prevent_default();
        }
    }
}

impl EntityInputHandler for TerminalView {
    fn text_for_range(
        &mut self,
        _: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        *actual = Some(0..self.composition.encode_utf16().count());
        Some(self.composition.clone())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let end = self.composition.encode_utf16().count();
        Some(UTF16Selection {
            range: end..end,
            reversed: false,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        (!self.composition.is_empty()).then(|| 0..self.composition.encode_utf16().count())
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.composition.clear();
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.composition.clear();
        self.selection = None;
        self.send(text.as_bytes().to_vec(), cx);
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        text: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.composition = text.to_owned();
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        _: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let (row, col) = DATA_CONTEXT
            .terminal
            .snapshot(self.id)
            .cursor
            .unwrap_or((0, 0));
        let metrics = self.metrics();
        Some(Bounds::new(
            point(
                bounds.left() + px(col as f32 * metrics.cell_width),
                bounds.top() + px(row as f32 * metrics.cell_height),
            ),
            size(px(metrics.cell_width), px(metrics.cell_height)),
        ))
    }
    fn character_index_for_point(
        &mut self,
        _: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(0)
    }
}
