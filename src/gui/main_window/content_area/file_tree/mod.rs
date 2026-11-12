mod core;
mod external;
mod ui;
use crate::gui::main_window::Workspace;
use crate::gui::main_window::ui::*;
use gpui_kit::component::input::InputState;
use std::{collections::HashSet, path::PathBuf};
pub(in crate::gui) struct State {
    pub(in crate::gui) search_window: Option<gpui_kit::AnyWindowHandle>,
    pub(in crate::gui) project_tree_scroll: UniformListScrollHandle,
    pub(in crate::gui) project_selection: HashSet<PathBuf>,
    pub(in crate::gui) project_tree_focus: gpui_kit::FocusHandle,
    pub(in crate::gui) project_batch_paths: Vec<PathBuf>,
    pub(in crate::gui) expanded_directories: HashSet<PathBuf>,
    pub(in crate::gui) loading_directories: HashSet<PathBuf>,
    pub(in crate::gui) moving_path: Option<PathBuf>,
    pub(in crate::gui) file_operation_running: bool,
    pub(in crate::gui) project_context_target: Option<(PathBuf, bool, bool)>,
    pub(in crate::gui) inline_file_edit: Option<InlineFileEdit>,
    pub(in crate::gui) tree_rows: Vec<TreeRow>,
    pub(in crate::gui) project_revision: u64,
    pub(in crate::gui) pending_reveal: Option<PathBuf>,
}

pub(crate) struct FileTree {
    pub(in crate::gui) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui) fn init_component_data(
    _window: &mut Window,
    cx: &mut App,
    _session: &Option<crate::domain::history::WorkspaceSession>,
    expanded: std::collections::HashSet<std::path::PathBuf>,
) -> State {
    State {
        search_window: None,
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
        pending_reveal: None,
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
    pub(in crate::gui) action: FileTreeMenuAction,
    pub(in crate::gui) source: PathBuf,
    pub(in crate::gui) directory: PathBuf,
    pub(in crate::gui) is_directory: bool,
    pub(in crate::gui) input: Entity<InputState>,
    pub(in crate::gui) error: Option<String>,
    pub(in crate::gui) _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
pub(in crate::gui) struct DraggedProjectEntry {
    pub(in crate::gui) paths: Vec<PathBuf>,
    pub(in crate::gui) path: PathBuf,
    pub(in crate::gui) name: String,
    pub(in crate::gui) is_directory: bool,
}

#[derive(Clone, Copy)]
pub(in crate::gui) enum FileTreeMenuAction {
    Run,
    OpenTerminal,
    NewFile,
    NewDirectory,
    Delete,
    Rename,
    Reveal,
    CopyProjectPath,
    CopyName,
    CopyAbsolutePath,
    Copy,
    Paste,
}

#[derive(Clone)]
pub(crate) enum Event {
    Initialize,
    OpenSearch,
    RevealFile(ProjectFile),
}

impl FileTree {
    pub(in crate::gui::main_window) fn new(
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
        self.subscription = Some(cx.spawn_in(window, async move |view, cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view
                    .update_in(cx, |this, window, cx| {
                        if let Some(event) = event {
                            this.handle_event(event, window, cx);
                        } else {
                            if this.state.project_revision != DATA_CONTEXT.project.file_tree.revision() {
                                this.rebuild_tree();
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
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
                Event::RevealFile(file) => self.reveal_file(file, window, cx),
            }
        }
    }
}
impl Render for FileTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_project_panel(cx)
    }
}
