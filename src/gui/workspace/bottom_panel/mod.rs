mod core;
mod external;
mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
use gpui_kit::component::input::InputState;
use std::collections::HashMap;
pub(in crate::gui::workspace) struct State {
    pub(in crate::gui::workspace) run_config_revision: u64,

    pub(in crate::gui::workspace) run_configuration_window: Option<AnyWindowHandle>,

    pub(in crate::gui::workspace) git_filter: Entity<InputState>,
    pub(in crate::gui::workspace) git_filter_subscription: Option<Subscription>,
    pub(in crate::gui::workspace) terminal_tabs: Vec<TerminalTab>,
    pub(in crate::gui::workspace) active_terminal: usize,
    pub(in crate::gui::workspace) terminal_sequence: u64,
    pub(in crate::gui::workspace) git_console_selected: bool,
    pub(in crate::gui::workspace) selected_git_commit: Option<String>,
    pub(in crate::gui::workspace) git_details_loading: bool,
    pub(in crate::gui::workspace) run_views: HashMap<
        crate::domain::terminal::TerminalId,
        Entity<crate::gui::component::terminal::TerminalView>,
    >,
    pub(in crate::gui::workspace) statistics_loading: bool,
    pub(in crate::gui::workspace) statistics_sort: usize,
    pub(in crate::gui::workspace) statistics_descending: bool,
    pub(in crate::gui::workspace) statistics_selected: Option<String>,
    pub(in crate::gui::workspace) statistics_scroll: UniformListScrollHandle,
    pub(in crate::gui::workspace) active_tool: ToolWindow,
    pub(in crate::gui::workspace) run_terminal:
        Entity<crate::gui::component::terminal::TerminalView>,
    pub(in crate::gui::workspace) git_loading: bool,
}

pub(crate) struct BottomPanel {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    window: &mut Window,
    cx: &mut App,
    session: &Option<crate::domain::history::WorkspaceSession>,
) -> State {
    State {
        run_config_revision: DATA_CONTEXT.project.run_configuration().request,

        run_configuration_window: None,

        git_filter: cx.new(|cx| InputState::new(window, cx).placeholder("Text or hash")),
        git_filter_subscription: None,
        terminal_tabs: vec![TerminalTab {
            id: crate::domain::terminal::TerminalId::Shell,
            label: "Local".into(),
            view: cx.new(|cx| {
                crate::gui::component::terminal::TerminalView::new(
                    crate::domain::terminal::TerminalId::Shell,
                    cx,
                )
            }),
        }],
        active_terminal: 0,
        terminal_sequence: 0,
        git_console_selected: false,
        selected_git_commit: None,
        git_details_loading: false,
        run_views: HashMap::new(),
        statistics_loading: false,
        statistics_sort: 0,
        statistics_descending: false,
        statistics_selected: None,
        statistics_scroll: UniformListScrollHandle::new(),
        active_tool: session
            .as_ref()
            .map_or(ToolWindow::Run, |saved| saved.active_tool),
        run_terminal: cx.new(|cx| {
            crate::gui::component::terminal::TerminalView::new(
                crate::domain::terminal::TerminalId::Run,
                cx,
            )
        }),
        git_loading: false,
    }
}

pub(in crate::gui) struct TerminalTab {
    pub(in crate::gui::workspace) id: crate::domain::terminal::TerminalId,
    pub(in crate::gui::workspace) label: String,
    pub(in crate::gui::workspace) view: Entity<crate::gui::component::terminal::TerminalView>,
}

pub(super) use crate::domain::history::ToolWindow;
impl ToolWindow {
    pub(in crate::gui::workspace) fn label(self) -> &'static str {
        match self {
            Self::Run => "运行",
            Self::Statistics => "统计",
            Self::Git => "Git",
            Self::Terminal => "终端",
        }
    }
}

#[derive(Clone)]
pub(crate) enum Event {
    Initialize,
    ToggleTool(ToolWindow),
    RefreshGit,
    OpenRunConfiguration,
    ResetRunAutomatic,
    StartExecution(crate::domain::run::RunAction),
    RunFile(std::path::PathBuf),
    StopRun,
    SetRunProfile(String),
}

impl BottomPanel {
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
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::BottomPanel);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
        );
        self.subscription = Some(cx.spawn_in(window, async move |view,cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view.update_in(cx,|this,window,cx| {
                    if let Some(event) = event { this.handle_event(event,window,cx); }
                    else { this.sync_run_configuration(window,cx); this.sync_run_views(cx); }
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
        if let crate::event_bus::gui_event::GuiEvent::BottomPanel(event) = event {
            match event {
                Event::Initialize => {
                    self.initialize(cx);
                }
                Event::ToggleTool(value) => {
                    self.toggle_tool(value, cx);
                }
                Event::RefreshGit => {
                    self.refresh_git(cx);
                }
                Event::OpenRunConfiguration => {
                    self.open_run_configuration(window, cx);
                }
                Event::ResetRunAutomatic => {
                    self.reset_run_automatic(window, cx);
                }
                Event::StartExecution(value) => {
                    self.start_execution(value, window, cx);
                }
                Event::RunFile(file) => self.run_file(file, cx),
                Event::StopRun => {
                    self.stop_run(cx);
                }
                Event::SetRunProfile(value) => {
                    self.set_run_profile(value, window, cx);
                }
            }
        }
    }
}
impl Render for BottomPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_tool_panel(cx)
    }
}
