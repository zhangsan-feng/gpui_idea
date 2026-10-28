mod external;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::watch;

/// Each component has its own lossless, coalescing redraw channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GuiComponent {
    LeftSidebar,
    RightSidebar,
    WindowBar,
    BottomBar,
    BottomPanel,
    FileTree,
    Editor,
    ProjectManagerWindow,
    RunConfiguration,
    Terminal(crate::domain::terminal::TerminalId),
}

#[derive(Default)]
pub(crate) struct RefreshGui {
    channels: Mutex<HashMap<GuiComponent, watch::Sender<u64>>>,
}
