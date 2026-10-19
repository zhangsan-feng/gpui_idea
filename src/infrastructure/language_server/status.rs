use crate::domain::language_server::LanguageServerStatus;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct StatusRegistry(Arc<Mutex<Registry>>);

struct Registry {
    next_id: u64,
    sessions: BTreeMap<u64, LanguageServerStatus>,
    output: watch::Sender<LanguageServerStatus>,
    closed: bool,
}

#[derive(Clone)]
pub(super) struct SessionStatus {
    registry: StatusRegistry,
    id: u64,
}

impl StatusRegistry {
    pub(super) fn new() -> Self {
        Self(Arc::new(Mutex::new(Registry {
            next_id: 0,
            sessions: BTreeMap::new(),
            output: watch::channel(LanguageServerStatus::Stopped).0,
            closed: false,
        })))
    }

    pub(super) fn session(&self) -> SessionStatus {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.next_id += 1;
        let id = state.next_id;
        state.sessions.insert(id, LanguageServerStatus::Stopped);
        SessionStatus {
            registry: self.clone(),
            id,
        }
    }

    pub(super) fn subscribe(&self) -> watch::Receiver<LanguageServerStatus> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .output
            .subscribe()
    }

    pub(super) fn shutdown(&self) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        state.closed = true;
        state.sessions.clear();
        state.output.send_replace(LanguageServerStatus::Stopped);
    }
}

impl SessionStatus {
    pub(super) fn send_replace(&self, next: LanguageServerStatus) {
        self.send_if_modified(|current| {
            if *current == next {
                return false;
            }
            *current = next;
            true
        });
    }

    pub(super) fn send_if_modified(&self, update: impl FnOnce(&mut LanguageServerStatus) -> bool) {
        let mut state = self
            .registry
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if state.closed {
            return;
        }
        let Some(current) = state.sessions.get_mut(&self.id) else {
            return;
        };
        let previous = current.clone();
        if !update(current) {
            return;
        }
        tracing::debug!(session = self.id, ?previous, next = ?current, "LSP session status transition");
        // Selection and publication share the lock: another session cannot
        // publish an older aggregate between computing and sending this value.
        let next = state
            .sessions
            .values()
            .max_by_key(|status| priority(status))
            .cloned()
            .unwrap_or(LanguageServerStatus::Stopped);
        state.output.send_if_modified(|current| {
            if *current == next {
                return false;
            }
            tracing::debug!(previous = ?current, ?next, "LSP aggregate status transition");
            *current = next;
            true
        });
    }
}

fn priority(status: &LanguageServerStatus) -> u8 {
    match status {
        LanguageServerStatus::Stopped => 0,
        LanguageServerStatus::Ready => 1,
        LanguageServerStatus::Warming => 2,
        LanguageServerStatus::Indexing => 3,
        LanguageServerStatus::Starting => 4,
        LanguageServerStatus::Preparing(_) => 5,
        LanguageServerStatus::Warning(_) => 6,
        LanguageServerStatus::Error(_) => 7,
    }
}
