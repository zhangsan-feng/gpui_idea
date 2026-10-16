mod core;
mod external;
mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
use std::{collections::HashMap, path::PathBuf};
pub(in crate::gui::workspace) struct State {
    pub(in crate::gui::workspace) project_windows:
        HashMap<crate::gui::project_manager::ProjectWindowKind, AnyWindowHandle>,
    pub(in crate::gui::workspace) settings_window: Option<AnyWindowHandle>,
    pub(in crate::gui::workspace) layout_subscriptions: Vec<Subscription>,
    pub(in crate::gui::workspace) persistence_timer: Option<Task<()>>,
    pub(in crate::gui::workspace) closing: bool,
    pub(in crate::gui::workspace) close_prompt_open: bool,
    pub(in crate::gui::workspace) session_ready: bool,
    pub(in crate::gui::workspace) session_revision: u64,
    pub(in crate::gui::workspace) window_view: crate::domain::history::WindowViewState,
    pub(in crate::gui::workspace) window_diagnostics:
        std::rc::Rc<std::cell::RefCell<crate::gui::workspace::ui::diagnostics::WindowDiagnostics>>,
    pub(in crate::gui::workspace) window_diagnostics_task: Option<Task<()>>,
    pub(in crate::gui::workspace) window_bounds_subscription: Option<Subscription>,
}

pub(crate) struct WindowBar {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    window: &mut Window,
    _cx: &mut App,
    session: &Option<crate::domain::history::WorkspaceSession>,
) -> State {
    State {
        project_windows: HashMap::new(),
        settings_window: None,
        layout_subscriptions: Vec::new(),
        persistence_timer: None,
        closing: false,
        close_prompt_open: false,
        session_ready: false,
        session_revision: session.as_ref().map_or(0, |saved| saved.revision),
        window_view: WindowBar::window_view_state(window),
        window_diagnostics: Default::default(),
        window_diagnostics_task: None,
        window_bounds_subscription: None,
    }
}

#[derive(Clone)]
pub(crate) enum ToolbarAction {
    OpenRecentProject(PathBuf),
    OpenProject,
    CreateProject,
    ManageProjects,
    Back,
    Forward,
    Search,
    Tool(ToolWindow),
    RefreshGit,
    CopyBranch,
    EditRun,
    AutoRun,
    Run(crate::domain::run::RunAction),
    Stop,
    CopyRunOutput,
    ClearRunOutput,
    ScrollRunToEnd,
    Profile(String),
    SelectRunConfiguration(u64),
    SelectRunInstance(crate::domain::terminal::TerminalId),
    CloseRunInstance(crate::domain::terminal::TerminalId),
    StopAllRunInstances,
    StopRunConfiguration(u64),
    RerunRunInstance(crate::domain::terminal::TerminalId),
}

#[derive(Clone)]
pub(crate) enum Event {
    Initialize,
    ScheduleSave,
    RequestClose,
    ToolbarAction(ToolbarAction),
}

impl WindowBar {
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
        this.start_subscribe(window, cx);
        this
    }
    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut refresh = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::WindowBar);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let crate::event_bus::gui_event::GuiEvent::WindowBar(event) = event {
            match event {
                Event::Initialize => {
                    self.initialize(window, cx);
                }
                Event::ScheduleSave => {
                    self.schedule_save(cx);
                }
                Event::RequestClose => {
                    self.request_close_core(window, cx);
                }
                Event::ToolbarAction(value) => {
                    self.toolbar_action(value, window, cx);
                }
            }
        }
    }
}
impl Render for WindowBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_toolbar(window, cx)
    }
}
