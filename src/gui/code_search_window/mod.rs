mod core;
mod external;
mod ui;

use crate::domain::project::ContentMatch;
use crate::gui::workspace::Workspace;
pub(in crate::gui) use external::open;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, Render, Subscription, WeakEntity, Window,
};

struct CodeSearchWindow {
    parent: WeakEntity<Workspace>,
    parent_window: gpui_kit::AnyWindowHandle,
    input: Entity<InputState>,
    request: u64,
    loading: bool,
    results: Vec<ContentMatch>,
    truncated: bool,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl CodeSearchWindow {
    fn new(
        parent: WeakEntity<Workspace>,
        parent_window: gpui_kit::AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("输入要搜索的代码内容…"));
        let subscriptions = vec![
            cx.observe(&input, |_, _, cx| cx.notify()),
            cx.subscribe_in(&input, window, |this, _, event, window, cx| match event {
                InputEvent::Change => this.search(cx),
                InputEvent::PressEnter { .. } if !this.loading => {
                    if let Some(result) = this.results.first().cloned() {
                        this.open_match(result, window, cx);
                    }
                }
                _ => {}
            }),
        ];
        input.update(cx, |input, cx| input.focus(window, cx));
        Self {
            parent,
            parent_window,
            input,
            request: 0,
            loading: false,
            results: Vec::new(),
            truncated: false,
            error: None,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for CodeSearchWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_view(cx)
    }
}
