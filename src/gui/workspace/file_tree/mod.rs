mod core;
mod external;
mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
use gpui_kit::component::input::InputState;
use std::{collections::HashSet, path::PathBuf};
pub(in crate::gui::workspace) struct State {
    pub(in crate::gui::workspace) search_input: Entity<InputState>,
    pub(in crate::gui::workspace) search_subscription: Option<Subscription>,
    pub(in crate::gui::workspace) search_results: Vec<crate::domain::project::ContentMatch>,
    pub(in crate::gui::workspace) search_loading: bool,
    pub(in crate::gui::workspace) search_request: u64,
    pub(in crate::gui::workspace) search_error: Option<String>,
    pub(in crate::gui::workspace) project_tree_scroll: UniformListScrollHandle,
    pub(in crate::gui::workspace) project_selection: HashSet<PathBuf>,
    pub(in crate::gui::workspace) project_tree_focus: gpui_kit::FocusHandle,
    pub(in crate::gui::workspace) project_batch_paths: Vec<PathBuf>,
    pub(in crate::gui::workspace) expanded_directories: HashSet<PathBuf>,
    pub(in crate::gui::workspace) loading_directories: HashSet<PathBuf>,
    pub(in crate::gui::workspace) moving_path: Option<PathBuf>,
    pub(in crate::gui::workspace) file_operation_running: bool,
    pub(in crate::gui::workspace) project_context_target: Option<(PathBuf, bool, bool)>,
    pub(in crate::gui::workspace) inline_file_edit: Option<InlineFileEdit>,
    pub(in crate::gui::workspace) tree_rows: Vec<TreeRow>,
    pub(in crate::gui::workspace) project_revision: u64,
}

pub(crate) struct FileTree {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    window: &mut Window,
    cx: &mut App,
    _session: &Option<crate::domain::history::WorkspaceSession>,
    expanded: std::collections::HashSet<std::path::PathBuf>,
) -> State {
    State {
        search_input: cx.new(|cx| InputState::new(window, cx).placeholder("输入要搜索的代码内容…")),
        search_subscription: None,
        search_results: Vec::new(),
        search_loading: false,
        search_request: 0,
        search_error: None,
        project_tree_scroll: UniformListScrollHandle::new(),
        project_selection: HashSet::new(),
        project_tree_focus: cx.focus_handle(),
        project_batch_paths: Vec::new(),
        expanded_directories: expanded,
        loading_directories: HashSet::new(),
        moving_path: None,
        file_operation_running: false,
        project_context_target: None,
        inline_file_edit: None,
        tree_rows: Vec::new(),
        project_revision: 0,
    }
}

#[derive(Clone)]
pub(in crate::gui) enum TreeRow {
    Entry(crate::domain::project::ProjectEntry, usize),
    InlineFile(usize),
    Message {
        directory: PathBuf,
        depth: usize,
        text: String,
        retry: bool,
    },
}

pub(in crate::gui) struct InlineFileEdit {
    pub(in crate::gui::workspace) action: FileTreeMenuAction,
    pub(in crate::gui::workspace) source: PathBuf,
    pub(in crate::gui::workspace) directory: PathBuf,
    pub(in crate::gui::workspace) is_directory: bool,
    pub(in crate::gui::workspace) input: Entity<InputState>,
    pub(in crate::gui::workspace) error: Option<String>,
    pub(in crate::gui::workspace) _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
pub(in crate::gui) struct DraggedProjectEntry {
    pub(in crate::gui::workspace) paths: Vec<PathBuf>,
    pub(in crate::gui::workspace) path: PathBuf,
    pub(in crate::gui::workspace) name: String,
    pub(in crate::gui::workspace) is_directory: bool,
}

#[derive(Clone, Copy)]
pub(in crate::gui) enum FileTreeMenuAction {
    Run,
    NewFile,
    NewDirectory,
    Delete,
    Rename,
    Reveal,
    CopyProjectPath,
    CopyAbsolutePath,
    Copy,
    Paste,
}

#[derive(Clone)]
pub(crate) enum Event {
    Initialize,
    OpenSearch,
}

impl FileTree {
    pub(super) fn new(
        workspace: gpui_kit::WeakEntity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let session = DATA_CONTEXT.startup().session.clone();
        let expanded = session
            .as_ref()
            .map(|s| s.expanded_directories.iter().cloned().collect())
            .unwrap_or_else(|| std::collections::HashSet::from([std::path::PathBuf::new()]));
        let mut this = Self {
            state: init_component_data(window, cx, &session, expanded),
            workspace,
            subscription: None,
        };
        this.start_subscribe(window, cx);
        this
    }
    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut refresh = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::FileTree);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
        );
        self.subscription = Some(cx.spawn_in(window, async move |view,cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view.update_in(cx,|this,window,cx| {
                    if let Some(event) = event { this.handle_event(event,window,cx); }
                    else { if this.state.project_revision != DATA_CONTEXT.project.revision() { this.rebuild_tree(); } }
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
        if let crate::event_bus::gui_event::GuiEvent::FileTree(event) = event {
            match event {
                Event::Initialize => {
                    self.initialize(window, cx);
                }
                Event::OpenSearch => {
                    self.open_file_search(window, cx);
                }
            }
        }
    }
}
impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_project_panel(cx)
    }
}
