mod core;
mod ui;

use super::SettingsWindow;
use crate::domain::theme::OnlineTheme;
use gpui_kit::{AppContext as _, Context, Entity, Window, component::input::InputState};

pub(super) struct ThemeBrowser {
    online: bool,
    busy: bool,
    query: Entity<InputState>,
    _query_subscription: gpui_kit::Subscription,
    searched_query: String,
    results: Vec<OnlineTheme>,
    offset: usize,
    next_offset: Option<usize>,
    page_offsets: Vec<usize>,
    loaded: bool,
    message: Option<String>,
}

impl ThemeBrowser {
    pub(super) fn new(window: &mut Window, cx: &mut Context<SettingsWindow>) -> Self {
        let query = cx.new(|cx| InputState::new(window, cx).placeholder("搜索 Open VSX 主题"));
        let subscription = cx.subscribe(&query, |this, _, event, cx| {
            if matches!(
                event,
                gpui_kit::component::input::InputEvent::PressEnter { .. }
            ) {
                this.search_themes(0, cx);
            }
        });
        Self {
            online: false,
            busy: false,
            query,
            _query_subscription: subscription,
            searched_query: String::new(),
            results: Vec::new(),
            offset: 0,
            next_offset: None,
            page_offsets: Vec::new(),
            loaded: false,
            message: None,
        }
    }

    pub(super) fn busy(&self) -> bool {
        self.busy
    }
}
