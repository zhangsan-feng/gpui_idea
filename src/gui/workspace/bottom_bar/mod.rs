mod external;
mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
pub(in crate::gui::workspace) struct State {
    pub(in crate::gui::workspace) cursor_editor: Option<gpui_kit::EntityId>,
    pub(in crate::gui::workspace) cursor_subscription: Option<gpui_kit::Subscription>,
    pub(in crate::gui::workspace) notice: Option<String>,
}

pub(crate) struct BottomBar {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    _window: &mut Window,
    _cx: &mut App,
    _session: &Option<crate::domain::history::WorkspaceSession>,
    notice: Option<String>,
) -> State {
    State {
        notice,
        cursor_editor: None,
        cursor_subscription: None,
    }
}

#[derive(Clone)]
pub(crate) enum Event {
    Notice(Option<String>),
    ClearNoticeIf(String),
}

impl BottomBar {
    pub(super) fn new(
        workspace: gpui_kit::WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let session = DATA_CONTEXT.startup().session.clone();
        let mut this = Self {
            state: init_component_data(window, cx, &session, DATA_CONTEXT.startup().notice.clone()),
            workspace,
            subscription: None,
        };
        this.start_subscribe(window, cx);
        this
    }
    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut refresh = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::BottomBar);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        );
        self.subscription = Some(cx.spawn_in(window, async move |view,cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view.update_in(cx,|this,window,cx| {
                    if let Some(event) = event { this.handle_event(event,window,cx); }
                    else { this.sync_cursor(cx); }
                    cx.notify();
                }).is_err() { break; }
            }
        }));
    }
    fn handle_event(
        &mut self,
        event: crate::event_bus::gui_event::GuiEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        if let crate::event_bus::gui_event::GuiEvent::BottomBar(event) = event {
            match event {
                Event::ClearNoticeIf(expected) => {
                    if self.state.notice.as_deref() == Some(expected.as_str()) {
                        self.state.notice = None;
                    }
                }
                Event::Notice(value) => {
                    self.state.notice = value;
                }
            }
        }
    }
}
impl Render for BottomBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_status_bar(cx)
    }
}
