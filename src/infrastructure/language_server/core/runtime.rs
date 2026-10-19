use super::{LanguageServerStatus, Readiness, Value};
use crate::infrastructure::language_server::status::SessionStatus;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;

#[derive(Clone)]
pub(super) struct Runtime(Arc<Mutex<State>>);

struct State {
    readiness: Readiness,
    progress: HashSet<String>,
    health: Option<LanguageServerStatus>,
    initialized: bool,
    warming: bool,
    stopped: bool,
    output: SessionStatus,
    changes: watch::Sender<Readiness>,
    inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
    server_id: u64,
}

impl Runtime {
    pub(super) fn new(
        rust: bool,
        output: SessionStatus,
        inventory: watch::Sender<Vec<crate::domain::language_server::LanguageServerInfo>>,
        server_id: u64,
    ) -> (Self, watch::Receiver<Readiness>) {
        let readiness = Readiness {
            server_status: rust,
            ..Readiness::default()
        };
        let (changes, receiver) = watch::channel(readiness);
        (
            Self(Arc::new(Mutex::new(State {
                readiness,
                progress: HashSet::new(),
                health: None,
                initialized: false,
                warming: false,
                stopped: false,
                output,
                changes,
                inventory,
                server_id,
            }))),
            receiver,
        )
    }

    fn update(&self, update: impl FnOnce(&mut State)) {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if state.stopped {
            return;
        }
        update(&mut state);
        if state.stopped {
            state.output.send_replace(LanguageServerStatus::Stopped);
            return;
        }
        // One lock covers notification, warmup and initialization state and
        // publication. No caller can publish a readiness snapshot taken earlier.
        state.changes.send_replace(state.readiness);
        let next = state.health.clone().unwrap_or_else(|| {
            if !state.initialized {
                LanguageServerStatus::Starting
            } else if state.readiness.busy()
                || (state.readiness.server_status && state.readiness.quiescent.is_none())
            {
                LanguageServerStatus::Indexing
            } else if state.warming {
                LanguageServerStatus::Warming
            } else {
                LanguageServerStatus::Ready
            }
        });
        state.output.send_replace(next.clone());
        state.inventory.send_if_modified(|servers| {
            let Some(server) = servers
                .iter_mut()
                .find(|server| server.id == state.server_id)
            else {
                return false;
            };
            if server.runtime_status.as_ref() == Some(&next) {
                return false;
            }
            tracing::debug!(
                server_id = state.server_id,
                language = server.language,
                name = server.name,
                ?next,
                "LSP inventory status transition"
            );
            server.runtime_status = Some(next);
            true
        });
    }

    pub(super) fn initialized(&self) {
        self.update(|state| state.initialized = true);
    }

    pub(super) fn warming(&self, warming: bool) {
        self.update(|state| state.warming = warming);
    }

    pub(super) fn failed(&self, detail: String) {
        self.update(|state| state.health = Some(LanguageServerStatus::Error(detail)));
    }

    pub(super) fn stop(&self) {
        self.update(|state| {
            state.stopped = true;
            state.warming = false;
        });
    }

    pub(super) fn notification(&self, message: &Value) {
        match message.get("method").and_then(Value::as_str) {
            Some("experimental/serverStatus") => self.update(|state| {
                if let Some(quiescent) = message.pointer("/params/quiescent").and_then(Value::as_bool) {
                    state.readiness.quiescent = Some(quiescent);
                    state.readiness.server_status = true;
                }
                let detail = message.pointer("/params/message").and_then(Value::as_str).unwrap_or_default();
                let health = message.pointer("/params/health").and_then(Value::as_str);
                state.health = match health {
                    Some("error") => Some(LanguageServerStatus::Error(detail.into())),
                    Some("warning") => Some(LanguageServerStatus::Warning(detail.into())),
                    _ => None,
                };
                tracing::debug!(?health, quiescent = ?state.readiness.quiescent, active = state.progress.len(), "LSP server readiness notification");
            }),
            Some("$/progress") => {
                let Some(token) = message.pointer("/params/token").filter(|token| token.is_string() || token.is_number()) else { return; };
                let kind = message.pointer("/params/value/kind").and_then(Value::as_str);
                if !matches!(kind, Some("begin" | "end")) { return; }
                let token = token.to_string();
                self.update(|state| {
                    if kind == Some("begin") {
                        state.progress.insert(token.clone());
                    } else {
                        state.progress.remove(&token);
                    }
                    state.readiness.progress = state.progress.len();
                    tracing::debug!(token, ?kind, active = state.progress.len(), value = %message["params"]["value"], "LSP work progress notification");
                });
            }
            _ => {}
        }
    }
}
