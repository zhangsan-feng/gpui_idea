mod core;
mod external;
mod process;
mod shell;
use crate::domain::terminal::TerminalId;
use alacritty_terminal::{
    Term,
    event::{Event, EventListener},
    event_loop::{EventLoop, EventLoopSender, State},
    sync::FairMutex,
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::mpsc;

#[derive(Clone)]
struct Listener {
    events: mpsc::UnboundedSender<Event>,
    exited: Arc<AtomicBool>,
}
impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        // Signal immediately: Application batches events, so its exit snapshot can
        // lag behind the PTY reader closing its resize/input channel.
        if matches!(&event, Event::ChildExit(_) | Event::Exit) {
            self.exited.store(true, Ordering::Release);
        }
        let _ = self.events.send(event);
    }
}
struct ShellLaunch {
    options: alacritty_terminal::tty::Options,
    title: String,
    #[cfg(windows)]
    _script: Option<shell::CommandScript>,
}

struct Session {
    stopped: bool,
    output_finalized: bool,
    completion_written: bool,
    cell_width: u16,
    cell_height: u16,
    pty_exited: Arc<AtomicBool>,
    process_id: u32,
    reader:
        Option<std::thread::JoinHandle<(EventLoop<alacritty_terminal::tty::Pty, Listener>, State)>>,
    _launch: ShellLaunch,
    command: Option<String>,
    exit_code: Option<i32>,
    generation: u64,
    term: Arc<FairMutex<Term<Listener>>>,
    sender: EventLoopSender,
    title: String,
    exited: bool,
}
pub(crate) struct TerminalStore {
    sessions: Mutex<HashMap<TerminalId, Session>>,
}
impl Drop for Session {
    fn drop(&mut self) {
        if self.reader.is_some() {
            if let Err(error) = process::terminate(self.process_id, &self.pty_exited) {
                tracing::warn!(%error, "terminal process cleanup failed");
                // Do not block the caller forever if the OS rejected termination.
                self.reader.take();
            }
        }
        // Alacritty owns the PTY and shell lifecycle on every platform.
        let _ = self
            .sender
            .send(alacritty_terminal::event_loop::Msg::Shutdown);
        if let Some(reader) = self.reader.take() {
            if reader.join().is_err() {
                tracing::warn!(
                    generation = self.generation,
                    "terminal reader panicked during cleanup"
                );
            }
        }
    }
}
impl TerminalStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}
