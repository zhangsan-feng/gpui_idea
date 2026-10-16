mod external;
use crate::event_bus::refresh_gui::GuiComponent;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::mpsc;

/// Commands carry intent; data updates are published through refresh_gui.
#[derive(Clone)]
pub(crate) enum GuiEvent {
    Editor(crate::gui::workspace::editor::Event),
    FileTree(crate::gui::workspace::file_tree::Event),
    WindowBar(crate::gui::workspace::window_bar::Event),
    BottomBar(crate::gui::workspace::bottom_bar::Event),
    BottomPanel(crate::gui::workspace::bottom_panel::Event),
    LeftSidebar(crate::gui::workspace::left_sidebar::Event),
}

/// A mailbox per workspace prevents commands leaking into another window.
#[derive(Default)]
pub(crate) struct GuiEventBus {
    mailboxes: Mutex<HashMap<(gpui_kit::EntityId, GuiComponent), mpsc::UnboundedSender<GuiEvent>>>,
}

pub(crate) struct Subscription {
    scope: gpui_kit::EntityId,
    target: GuiComponent,
    pub(crate) receiver: mpsc::UnboundedReceiver<GuiEvent>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        crate::event_bus::EVENT_BUS
            .gui_event
            .mailboxes
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&(self.scope, self.target));
    }
}

impl GuiEvent {
    fn target(&self) -> GuiComponent {
        match self {
            Self::Editor(..) => GuiComponent::Editor,
            Self::FileTree(..) => GuiComponent::FileTree,
            Self::WindowBar(..) => GuiComponent::WindowBar,
            Self::BottomBar(..) => GuiComponent::BottomBar,
            Self::BottomPanel(..) => GuiComponent::BottomPanel,
            Self::LeftSidebar(..) => GuiComponent::LeftSidebar,
        }
    }
}
