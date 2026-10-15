mod core;
mod ui;
use crate::{data_context::DATA_CONTEXT, domain::terminal::TerminalId};
use gpui_kit::{Context, FocusHandle, Focusable, IntoElement, Render, Window};
gpui_kit::actions!(terminal, [SendTab, CopyTerminal, PasteTerminal]);
pub(crate) fn init(cx: &mut gpui_kit::App) {
    cx.bind_keys([
        gpui_kit::KeyBinding::new("tab", SendTab, Some("Terminal")),
        gpui_kit::KeyBinding::new("ctrl-shift-c", CopyTerminal, Some("Terminal")),
        gpui_kit::KeyBinding::new("ctrl-shift-v", PasteTerminal, Some("Terminal")),
    ]);
}

pub(crate) struct TerminalView {
    scrollbar_bounds: Option<gpui_kit::Bounds<gpui_kit::Pixels>>,
    scrollbar_grab: Option<f32>,
    id: TerminalId,
    focus: FocusHandle,
    dimensions: (usize, usize),
    generation: u64,
    starting: bool,
    started: bool,
    composition: String,
    selection: Option<((usize, usize), (usize, usize))>,
    dragging: bool,
    bounds: Option<gpui_kit::Bounds<gpui_kit::Pixels>>,
    error: Option<String>,
    input: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    operation_task: Option<gpui_kit::Task<()>>,
    focus_pending: bool,
}
impl TerminalView {
    pub(crate) fn new(id: TerminalId, cx: &mut Context<Self>) -> Self {
        let (input, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        cx.spawn(async move |_, _| {
            let _ = crate::application::lifecycle::run_application(async move {
                while let Some(bytes) = receiver.recv().await {
                    if let Err(error) = crate::application::APPLICATION
                        .terminal
                        .input(id, bytes)
                        .await
                    {
                        tracing::warn!(?id, %error, "terminal input failed");
                    }
                }
                Ok(())
            })
            .await;
        })
        .detach();
        let mut refresh = crate::data_context::DATA_CONTEXT.subscribe_changes();
        cx.spawn(async move |view, cx| {
            while refresh.changed().await.is_ok() {
                if view
                    .update(cx, |this, cx| {
                        let snapshot = DATA_CONTEXT.terminal.snapshot(this.id);
                        if snapshot.generation != this.generation {
                            this.generation = snapshot.generation;
                            this.dimensions = (0, 0);
                            this.selection = None;
                            this.composition.clear();
                            if snapshot.generation > 0 && this.id.is_run() {
                                this.focus_pending = true;
                            }
                        }
                        if snapshot.generation > 0 {
                            this.started = true;
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            scrollbar_bounds: None,
            scrollbar_grab: None,
            id,
            focus: cx.focus_handle(),
            dimensions: (0, 0),
            generation: DATA_CONTEXT.terminal.snapshot(id).generation,
            starting: false,
            started: DATA_CONTEXT.terminal.snapshot(id).generation > 0,
            composition: String::new(),
            selection: None,
            dragging: false,
            bounds: None,
            error: None,
            input,
            operation_task: None,
            focus_pending: matches!(id, TerminalId::RunInstance(_)),
        }
    }
}

#[derive(Clone, Copy)]
struct TerminalMetrics {
    font: &'static str,
    font_size: f32,
    cell_width: f32,
    cell_height: f32,
}

impl TerminalView {
    fn metrics(&self) -> TerminalMetrics {
        if self.id.is_run() {
            TerminalMetrics {
                font: "Consolas",
                font_size: 14.,
                cell_width: 8.,
                cell_height: 22.,
            }
        } else {
            TerminalMetrics {
                font: "Cascadia Mono",
                font_size: 13.333,
                cell_width: 8.,
                cell_height: 18.,
            }
        }
    }
}
impl Focusable for TerminalView {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}
impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_pending {
            self.focus_pending = false;
            self.focus.focus(window, cx);
        }
        self.render_terminal(cx)
    }
}
