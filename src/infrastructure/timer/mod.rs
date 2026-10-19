mod core;
mod external;
mod operations;

use crate::event_bus::{FileChanges, timer_event::TimerEvent};
use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::oneshot, task::JoinHandle, time::Instant};

pub(crate) struct TimerPipeline {
    state: Mutex<Option<JoinHandle<()>>>,
}

struct Scheduled {
    deadline: Instant,
    event: ScheduledEvent,
}

enum ScheduledEvent {
    Delay {
        name: &'static str,
        completion: oneshot::Sender<()>,
    },
    WatchFiles {
        root: PathBuf,
        pending: Arc<Mutex<FileChanges>>,
        period: Duration,
    },
}

impl TimerPipeline {
    pub(super) fn start_subscribe(&self) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.is_some() {
            return Ok(());
        }
        tokio::runtime::Handle::try_current().context("定时事件必须在 Tokio runtime 中订阅")?;
        let events = crate::event_bus::EVENT_BUS.timer_event.subscribe()?;
        *state = Some(tokio::spawn(core::run(events)));
        tracing::info!("timer subscribed to event_bus.timer_event");
        Ok(())
    }

    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(None),
        }
    }
}
