mod external;

use super::FileChanges;
use crate::{domain::document::DocumentSnapshot, infrastructure::LocalChangeGuard};
use anyhow::Result;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, atomic::AtomicU64},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

/// Data-only requests. The timer subscriber chooses the handler; callers cannot
/// submit closures or executable futures through the event bus.
pub(crate) enum TimerEvent {
    Delay {
        id: u64,
        name: &'static str,
        delay: Duration,
        completion: oneshot::Sender<()>,
    },
    WatchFiles {
        id: u64,
        root: PathBuf,
        pending: Arc<Mutex<FileChanges>>,
        period: Duration,
    },
    SaveFile {
        id: u64,
        document: Arc<DocumentSnapshot>,
        completion: oneshot::Sender<Result<Arc<[u8]>>>,
    },
    ExcludeGuiFileChange {
        id: u64,
        root: PathBuf,
        paths: Vec<PathBuf>,
        completion: oneshot::Sender<Result<LocalChangeGuard>>,
    },
    Cancel(u64),
}

pub(crate) struct TimerRegistration {
    id: u64,
    sender: mpsc::UnboundedSender<TimerEvent>,
}

impl Drop for TimerRegistration {
    fn drop(&mut self) {
        let _ = self.sender.send(TimerEvent::Cancel(self.id));
    }
}

/// A single consumer queue preserves save/exclusion requests when events burst.
pub(crate) struct TimerEventBus {
    sender: mpsc::UnboundedSender<TimerEvent>,
    receiver: Mutex<Option<mpsc::UnboundedReceiver<TimerEvent>>>,
    next_id: AtomicU64,
}

impl Default for TimerEventBus {
    fn default() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Mutex::new(Some(receiver)),
            next_id: AtomicU64::new(1),
        }
    }
}
