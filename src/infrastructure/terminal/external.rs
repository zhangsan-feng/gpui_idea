use super::*;
use crate::domain::terminal::TerminalSnapshot;
use alacritty_terminal::{event::WindowSize, event_loop::Msg, grid::Scroll, term::test::TermSize};
use anyhow::{Context, Result, anyhow};
use std::{borrow::Cow, path::PathBuf};

impl TerminalStore {
    pub(crate) async fn release(&'static self, id: TerminalId) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let retired = { self.sessions.lock().unwrap().remove(&id) };
            // Never destroy a ConPTY session while holding the shared session lock.
            drop(retired);
        })
        .await
        .context("terminal release task failed")?;
        Ok(())
    }
    pub(crate) async fn start(
        &'static self,
        id: TerminalId,
        root: PathBuf,
        command: Option<String>,
    ) -> Result<Option<(u64, mpsc::UnboundedReceiver<Event>)>> {
        tokio::task::spawn_blocking(move || {
            let mut sessions = self.sessions.lock().unwrap();
            if sessions.get(&id).is_some_and(|session| !session.exited) {
                return Ok(None);
            }
            let (session, rx) = core::start(root, command)?;
            tracing::info!(?id, "alacritty PTY session started");
            let generation = session.generation;
            sessions.insert(id, session);
            Ok(Some((generation, rx)))
        })
        .await
        .context("terminal startup task failed")?
    }
    pub(crate) fn input(&self, id: TerminalId, bytes: Vec<u8>) -> Result<bool> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.get(&id).ok_or_else(|| anyhow!("终端尚未启动"))?;
        if session.exited || session.pty_exited.load(Ordering::Acquire) {
            return Ok(false);
        }
        let scrolled = {
            let mut term = session.term.lock();
            let scrolled = term.grid().display_offset() != 0;
            if scrolled {
                term.scroll_display(Scroll::Bottom);
            }
            scrolled
        };
        session
            .sender
            .send(Msg::Input(Cow::Owned(bytes)))
            .context("terminal input failed")?;
        Ok(scrolled)
    }
    pub(crate) fn resize(
        &self,
        id: TerminalId,
        columns: usize,
        lines: usize,
        cell_width: u16,
        cell_height: u16,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&id) {
            session.cell_width = cell_width;
            session.cell_height = cell_height;
            session.term.lock().resize(TermSize::new(columns, lines));
            if session.exited || session.pty_exited.load(Ordering::Acquire) {
                tracing::debug!(
                    ?id,
                    generation = session.generation,
                    columns,
                    lines,
                    "completed terminal output grid resized; PTY resize skipped"
                );
                return Ok(());
            }
            if let Err(error) = session.sender.send(Msg::Resize(WindowSize {
                num_cols: columns as u16,
                num_lines: lines as u16,
                cell_width,
                cell_height,
            })) {
                // The child can exit between the liveness check and channel send.
                if session.pty_exited.load(Ordering::Acquire) {
                    tracing::debug!(
                        ?id,
                        generation = session.generation,
                        columns,
                        lines,
                        "terminal exited during PTY resize; output grid retained"
                    );
                    return Ok(());
                }
                tracing::error!(
                    ?id,
                    generation = session.generation,
                    columns,
                    lines,
                    ?error,
                    "active terminal PTY resize send failed"
                );
                return Err(error).context("terminal resize failed");
            }
            tracing::debug!(
                ?id,
                generation = session.generation,
                columns,
                lines,
                "terminal grid resized and PTY resize queued"
            );
        }
        Ok(())
    }
    pub(crate) fn scroll(&self, id: TerminalId, lines: i32) {
        if let Some(session) = self.sessions.lock().unwrap().get(&id) {
            session.term.lock().scroll_display(Scroll::Delta(lines));
        }
    }
    pub(crate) fn scroll_to(&self, id: TerminalId, offset: usize) {
        use alacritty_terminal::grid::Dimensions;
        if let Some(session) = self.sessions.lock().unwrap().get(&id) {
            let mut term = session.term.lock();
            let offset = offset.min(term.grid().history_size());
            let delta = offset as i32 - term.grid().display_offset() as i32;
            term.scroll_display(Scroll::Delta(delta));
        }
    }
    pub(crate) async fn stop(&'static self, id: TerminalId) -> Result<()> {
        tokio::task::spawn_blocking(move || {
            let mut sessions = self.sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(&id) {
                session.process_job.terminate()?;
                if !session.exited {
                    session.stopped = true;
                    session.output_finalized = false;
                    session.exit_code = Some(130);
                }
                let _ = session.sender.send(Msg::Shutdown);
                // Stop cancels the reader and can discard pending tail output.
                // Only the reader's drained Event::Exit may finalize a transcript.
                session.exited = true;
                tracing::info!(?id, "terminal process tree stopped");
            }
            Ok(())
        })
        .await
        .context("terminal stop task failed")?
    }
    pub(crate) fn clear(&self, id: TerminalId) {
        use alacritty_terminal::vte::ansi::{ClearMode, Handler};
        if let Some(session) = self.sessions.lock().unwrap().get(&id) {
            let mut term = session.term.lock();
            term.clear_screen(ClearMode::Saved);
            term.clear_screen(ClearMode::All);
        }
    }
    pub(crate) async fn update(
        &'static self,
        id: TerminalId,
        generation: Option<u64>,
        events: Vec<Event>,
    ) -> Result<TerminalSnapshot> {
        tokio::task::spawn_blocking(move || {
            let mut sessions = self.sessions.lock().unwrap();
            let session = sessions
                .get_mut(&id)
                .ok_or_else(|| anyhow!("终端尚未启动"))?;
            if generation.is_some_and(|generation| generation != session.generation) {
                return Err(anyhow!("终端会话已替换"));
            }
            for event in events {
                match event {
                    Event::Title(title) => session.title = title,
                    Event::PtyWrite(text) => {
                        let _ = session.sender.send(Msg::Input(text.into_bytes().into()));
                    }
                    Event::TextAreaSizeRequest(format) => {
                        use alacritty_terminal::grid::Dimensions;
                        let term = session.term.lock();
                        let text = format(WindowSize {
                            num_cols: term.columns() as u16,
                            num_lines: term.screen_lines() as u16,
                            cell_width: session.cell_width,
                            cell_height: session.cell_height,
                        });
                        let _ = session.sender.send(Msg::Input(text.into_bytes().into()));
                    }
                    Event::ColorRequest(index, format) => {
                        let term = session.term.lock();
                        let color = core::indexed_color(index, term.renderable_content().colors);
                        let text = format(alacritty_terminal::vte::ansi::Rgb {
                            r: (color >> 16) as u8,
                            g: (color >> 8) as u8,
                            b: color as u8,
                        });
                        let _ = session.sender.send(Msg::Input(text.into_bytes().into()));
                    }
                    Event::ChildExit(status) => {
                        if !session.stopped {
                            session.exit_code = status.code();
                        }
                        // Event::Exit follows only after the remaining PTY output
                        // has been drained. Keep consuming until that final event.
                        tracing::info!(?id,code=?status.code(),"terminal process exited");
                    }
                    Event::Exit => {
                        session.exited = true;
                        session.output_finalized = true;
                        tracing::info!(?id, "terminal child exited");
                    }
                    _ => {}
                }
            }
            core::finish_output(session);
            Ok(core::snapshot(session))
        })
        .await
        .context("terminal snapshot task failed")?
    }
    pub(crate) async fn shutdown(&'static self) {
        let ids = self
            .sessions
            .lock()
            .unwrap()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for id in ids {
            if let Err(error) = self.stop(id).await {
                tracing::warn!(?id,%error,"terminal shutdown failed");
            }
        }
    }
}
