mod scrollbar;
use super::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl TerminalView {
    pub(super) fn render_terminal(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let metrics = self.metrics();
        let entity = cx.entity();
        let snapshot = DATA_CONTEXT.terminal.snapshot(self.id);
        let selection = self.selection.map(|(a, b)| (a.min(b), a.max(b)));
        let composition = self.composition.clone();
        let focus = self.focus.clone();
        let error = self.error.clone();
        div()
            .id("terminal-surface")
            .relative()
            .flex()
            .track_focus(&self.focus)
            .key_context("Terminal")
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(rgb(0xffffff))
            .cursor_text()
            .on_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(|this, _: &SendTab, _, cx| this.send_tab(cx)))
            .on_action(cx.listener(|this, _: &CopyTerminal, _, cx| this.copy(cx)))
            .on_action(cx.listener(|this, _: &PasteTerminal, _, cx| this.paste_clipboard(cx)))
            .on_scroll_wheel(
                cx.listener(|this, event: &ScrollWheelEvent, _, cx| this.scroll(event, cx)),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.focus.focus(window, cx);
                    let cell = this.cell_at(event.position);
                    this.selection = Some((cell, cell));
                    this.dragging = true;
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.scrollbar_grab.is_some() {
                    this.drag_scrollbar(event.position.y, cx);
                    return;
                }
                if this.dragging {
                    let cell = this.cell_at(event.position);
                    if let Some((start, _)) = this.selection {
                        this.selection = Some((start, cell));
                        cx.notify();
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = false;
                    this.scrollbar_grab = None;
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = false;
                    this.scrollbar_grab = None;
                }),
            )
            .child(
                canvas(
                    move |bounds, _, cx| {
                        entity.update(cx, |this, cx| this.resize(bounds, cx));
                    },
                    {
                        let entity = cx.entity();
                        move |bounds, _, window, cx| {
                            window.handle_input(
                                &focus,
                                ElementInputHandler::new(bounds, entity.clone()),
                                cx,
                            );
                            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                                for (index, cell) in snapshot.cells.iter().enumerate() {
                                    let row = index / snapshot.columns;
                                    let col = index % snapshot.columns;
                                    let origin = point(
                                        bounds.left() + px(col as f32 * metrics.cell_width),
                                        bounds.top() + px(row as f32 * metrics.cell_height),
                                    );
                                    let selected = selection.is_some_and(|(start, end)| {
                                        (row, col) >= start && (row, col) <= end && start != end
                                    });
                                    if cell.background != 0xffffff || selected {
                                        window.paint_quad(fill(
                                            Bounds::new(
                                                origin,
                                                size(
                                                    px(metrics.cell_width
                                                        * if cell.wide { 2. } else { 1. }),
                                                    px(metrics.cell_height),
                                                ),
                                            ),
                                            rgb(if selected { 0xc4dcff } else { cell.background }),
                                        ));
                                    }
                                    if cell.text.trim().is_empty() {
                                        continue;
                                    }
                                    let mut font = font(metrics.font);
                                    if cell.bold {
                                        font.weight = FontWeight::BOLD;
                                    }
                                    if cell.italic {
                                        font.style = FontStyle::Italic;
                                    }
                                    let run = TextRun {
                                        len: cell.text.len(),
                                        font,
                                        color: rgb(cell.foreground).into(),
                                        background_color: None,
                                        underline: None,
                                        strikethrough: None,
                                    };
                                    let line = window.text_system().shape_line(
                                        cell.text.clone().into(),
                                        px(metrics.font_size),
                                        &[run],
                                        None,
                                    );
                                    let _ = line.paint(
                                        origin,
                                        px(metrics.cell_height),
                                        TextAlign::Left,
                                        None,
                                        window,
                                        cx,
                                    );
                                    if cell.underline {
                                        window.paint_quad(fill(
                                            Bounds::new(
                                                point(
                                                    origin.x,
                                                    origin.y + px(metrics.cell_height - 2.),
                                                ),
                                                size(px(metrics.cell_width), px(1.)),
                                            ),
                                            rgb(cell.foreground),
                                        ));
                                    }
                                }
                                if let Some((row, col)) = snapshot.cursor {
                                    let origin = point(
                                        bounds.left() + px(col as f32 * metrics.cell_width),
                                        bounds.top() + px(row as f32 * metrics.cell_height),
                                    );
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            origin,
                                            size(
                                                px(if focus.is_focused(window) { 2. } else { 1. }),
                                                px(metrics.cell_height),
                                            ),
                                        ),
                                        rgb(0x3574f0),
                                    ));
                                    if !composition.is_empty() {
                                        let run = TextRun {
                                            len: composition.len(),
                                            font: font(metrics.font),
                                            color: rgb(0x20242b).into(),
                                            background_color: Some(rgb(0xffffff).into()),
                                            underline: None,
                                            strikethrough: None,
                                        };
                                        let line = window.text_system().shape_line(
                                            composition.clone().into(),
                                            px(metrics.font_size),
                                            &[run],
                                            None,
                                        );
                                        let _ = line.paint(
                                            origin,
                                            px(metrics.cell_height),
                                            TextAlign::Left,
                                            None,
                                            window,
                                            cx,
                                        );
                                    }
                                }
                            });
                        }
                    },
                )
                .flex_1()
                .h_full()
                .min_w_0(),
            )
            .when(self.id.is_run(), |view| {
                view.child(self.render_scrollbar(cx))
            })
            .when_some(error, |view, error| {
                view.child(
                    div()
                        .absolute()
                        .top_2()
                        .left_2()
                        .text_color(rgb(0xc42b35))
                        .child(error),
                )
            })
    }
}
