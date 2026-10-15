mod core;
mod external;
mod process;
use crate::domain::terminal::TerminalId;
use alacritty_terminal::{
    Term,
    event::{Event, EventListener},
    event_loop::EventLoopSender,
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
struct Session {
    stopped: bool,
    output_finalized: bool,
    completion_written: bool,
    cell_width: u16,
    cell_height: u16,
    pty_exited: Arc<AtomicBool>,
    process_job: process::ProcessJob,
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
impl TerminalStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}
