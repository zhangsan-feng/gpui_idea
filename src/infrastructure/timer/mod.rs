mod core;
mod external;

use anyhow::Result;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::Duration,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::{AbortHandle, JoinHandle},
    time::Instant,
};

type Operation = Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;
pub(crate) struct TimerPipeline {
    state: Mutex<Option<PipelineState>>,
    next_id: AtomicU64,
}
struct PipelineState {
    commands: mpsc::UnboundedSender<Command>,
    task: JoinHandle<()>,
}
enum Command {
    Add {
        id: u64,
        name: &'static str,
        delay: Duration,
        work: Work,
    },
    Cancel(u64),
}
enum Work {
    Delay(oneshot::Sender<()>),
    Repeating {
        period: Duration,
        operation: Operation,
    },
}
struct Scheduled {
    name: &'static str,
    deadline: Option<Instant>,
    running: Option<AbortHandle>,
    work: Work,
}

/// Dropping a registration cancels future ticks and an in-flight operation.
pub(crate) struct TimerRegistration {
    id: u64,
    commands: mpsc::UnboundedSender<Command>,
}
impl Drop for TimerRegistration {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Cancel(self.id));
    }
}
impl TimerPipeline {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(None),
            next_id: AtomicU64::new(1),
        }
    }
}
