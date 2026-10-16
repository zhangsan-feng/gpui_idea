mod core;
mod external;
pub(crate) mod gui_event;
pub(crate) mod refresh_gui;

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock},
};
use tokio::sync::broadcast;

pub(crate) static EVENT_BUS: LazyLock<EventBus> = LazyLock::new(EventBus::new);

#[derive(Default, Debug)]
pub(crate) struct FileChanges {
    pub(crate) paths: HashSet<PathBuf>,
    pub(crate) rescan: bool,
    pub(crate) event_count: u64,
}

/// A request to reload affected snapshots, not a notification that data is ready.
#[derive(Debug)]
pub(crate) struct RefreshDataContext {
    pub(crate) root: PathBuf,
    pub(crate) changes: FileChanges,
}

pub(crate) struct EventBus {
    refresh_data_context: broadcast::Sender<Arc<RefreshDataContext>>,
    pub(crate) refresh_gui: refresh_gui::RefreshGui,
    pub(crate) gui_event: gui_event::GuiEventBus,
}

impl EventBus {
    fn new() -> Self {
        Self {
            refresh_data_context: broadcast::channel(64).0,
            refresh_gui: Default::default(),
            gui_event: Default::default(),
        }
    }
}
