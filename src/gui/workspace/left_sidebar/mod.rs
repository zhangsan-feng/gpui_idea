mod external;
use gpui_kit::component::ResizableState;
mod core;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
mod ui;

pub(crate) struct LeftSidebar {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    _window: &mut Window,
    cx: &mut App,
    session: &Option<crate::domain::history::WorkspaceSession>,
) -> State {
    tracing::debug!(
        project_width = ?session.as_ref().and_then(|saved| saved.project_width),
        run_height = ?session.as_ref().and_then(|saved| saved.run_height),
        "workspace panel dimensions loaded for restoration"
    );
    State {
        layout_subscriptions: Vec::new(),
        project_panel_open: DATA_CONTEXT.has_project()
            && session
                .as_ref()
                .is_none_or(|saved| saved.project_panel_open),
        run_panel_open: DATA_CONTEXT.has_project()
            && session.as_ref().is_none_or(|saved| saved.run_panel_open),
        project_resize: cx.new(|_| ResizableState::default()),
        run_resize: cx.new(|_| ResizableState::default()),
    }
}

pub(in crate::gui::workspace) struct State {
    layout_subscriptions: Vec<Subscription>,
    pub(in crate::gui::workspace) project_panel_open: bool,
    pub(in crate::gui::workspace) run_panel_open: bool,
    pub(in crate::gui::workspace) project_resize: Entity<ResizableState>,
    pub(in crate::gui::workspace) run_resize: Entity<ResizableState>,
}

#[derive(Clone)]
pub(crate) enum Event {
    ToggleProjectPanel,
    SetRunPanel(bool),
}

impl LeftSidebar {
    pub(super) fn new(
        workspace: gpui_kit::WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let session = DATA_CONTEXT.startup().session.clone();
        let mut this = Self {
            state: init_component_data(window, cx, &session),
            workspace,
            subscription: None,
        };
        this.state.layout_subscriptions.push(
            cx.observe(&this.state.project_resize, |this, _, cx| {
                this.notify_layout(cx)
            }),
        );
        this.state
            .layout_subscriptions
            .push(cx.observe(&this.state.run_resize, |this, _, cx| this.notify_layout(cx)));
        this.start_subscribe(window, cx);
        this
    }
    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut refresh = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::LeftSidebar);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
        );
        self.subscription = Some(cx.spawn_in(window, async move |view,cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view.update_in(cx,|this,window,cx| {
                    if let Some(event) = event { this.handle_event(event,window,cx); }
                    else {  }
                    cx.notify();
                }).is_err() { break; }
            }
        }));
    }
    fn handle_event(
        &mut self,
        event: crate::event_bus::gui_event::GuiEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let crate::event_bus::gui_event::GuiEvent::LeftSidebar(event) = event {
            match event {
                Event::ToggleProjectPanel => {
                    self.toggle_project_panel(cx);
                }
                Event::SetRunPanel(value) => {
                    self.state.run_panel_open = value;
                    self.notify_layout(cx);
                }
            }
        }
    }
}
impl Render for LeftSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_activity_bar(cx)
    }
}
