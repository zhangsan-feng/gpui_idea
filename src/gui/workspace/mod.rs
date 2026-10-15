use crate::data_context::DATA_CONTEXT;
mod core;
mod external;
mod run_editor;
mod ui;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use gpui_kit::component::ResizableState;
use gpui_kit::component::input::EditorState;
use gpui_kit::component::input::InputState;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Context, Entity, IntoElement, Render, ScrollHandle,
    Subscription, Task, UniformListScrollHandle, Window,
};

gpui_kit::actions!(workspace, [SaveDocument, CloseAllDocuments]);

pub(super) use crate::domain::history::ToolWindow;
impl ToolWindow {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Run => "运行",
            Self::Statistics => "统计",
            Self::Git => "Git",
            Self::Terminal => "终端",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct ProjectFile(pub PathBuf);
impl ProjectFile {
    pub(super) fn name(&self) -> String {
        self.0
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
    pub(super) fn relative_path(&self) -> &std::path::Path {
        &self.0
    }
    pub(super) fn path(&self, root: &std::path::Path) -> PathBuf {
        root.join(&self.0)
    }
}

#[derive(Clone)]
pub(super) enum TreeRow {
    Entry(crate::domain::project::ProjectEntry, usize),
    InlineFile(usize),
    Message {
        directory: PathBuf,
        depth: usize,
        text: String,
        retry: bool,
    },
}

pub(super) struct InlineFileEdit {
    pub(super) action: FileTreeMenuAction,
    pub(super) source: PathBuf,
    pub(super) directory: PathBuf,
    pub(super) is_directory: bool,
    pub(super) input: Entity<InputState>,
    pub(super) error: Option<String>,
    pub(super) _subscriptions: Vec<Subscription>,
}

#[derive(Clone)]
pub(super) struct DraggedEditorTab {
    pub(super) file: ProjectFile,
    pub(super) label: String,
}

#[derive(Clone)]
pub(super) struct DraggedProjectEntry {
    pub(super) path: PathBuf,
    pub(super) name: String,
    pub(super) is_directory: bool,
}

#[derive(Clone, Copy)]
pub(super) enum FileTreeMenuAction {
    NewFile,
    NewDirectory,
    Delete,
    Rename,
    Reveal,
    Copy,
    Paste,
}

pub(super) struct TerminalTab {
    pub(super) id: crate::domain::terminal::TerminalId,
    pub(super) label: String,
    pub(super) view: Entity<crate::gui::terminal::TerminalView>,
}

#[derive(Clone)]
pub(super) enum ToolbarAction {
    OpenProject,
    CreateProject,
    ManageProjects,
    OpenRecentProject(PathBuf),
    Back,
    Forward,
    Save,
    Search,
    ToggleProject,
    Tool(ToolWindow),
    RefreshProject,
    RefreshGit,
    CopyProjectPath,
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
    CloseTabs,
    CloseWindow,
}

pub struct Workspace {
    pub(super) project_windows:
        HashMap<crate::gui::project_manager::ProjectWindowKind, AnyWindowHandle>,
    pub(super) run_configuration_window: Option<AnyWindowHandle>,
    pub(super) navigation: Vec<ProjectFile>,
    pub(super) navigation_index: usize,
    pub(super) navigating: bool,
    pub(super) search_input: Entity<InputState>,
    pub(super) search_subscription: Option<Subscription>,
    pub(super) search_results: Vec<crate::domain::project::ContentMatch>,
    pub(super) pending_search_positions: HashMap<PathBuf, gpui_kit::component::input::Position>,
    pub(super) search_loading: bool,
    pub(super) search_request: u64,
    pub(super) search_error: Option<String>,
    pub(super) layout_subscriptions: Vec<Subscription>,
    pub(super) git_filter: Entity<InputState>,
    pub(super) git_filter_subscription: Option<Subscription>,
    pub(super) terminal_tabs: Vec<TerminalTab>,
    pub(super) active_terminal: usize,
    pub(super) terminal_sequence: u64,
    pub(super) git_console_selected: bool,
    pub(super) selected_git_commit: Option<String>,
    pub(super) git_details_loading: bool,
    pub(super) run_config_revision: u64,
    pub(super) run_views:
        HashMap<crate::domain::terminal::TerminalId, Entity<crate::gui::terminal::TerminalView>>,
    pub(super) statistics_loading: bool,
    pub(super) statistics_sort: usize,
    pub(super) statistics_descending: bool,
    pub(super) statistics_selected: Option<String>,
    pub(super) active_tool: ToolWindow,
    pub(super) run_terminal: Entity<crate::gui::terminal::TerminalView>,
    pub(super) git_loading: bool,
    pub(super) editors: HashMap<ProjectFile, Entity<EditorState>>,
    pub(super) definition_request: u64,
    pub(super) definition_observer: Option<Subscription>,
    pub(super) external_source_files: HashSet<ProjectFile>,
    pub(super) editor_revisions: HashMap<ProjectFile, u64>,
    pub(super) syncing_editors: HashSet<ProjectFile>,
    pub(super) editor_subscriptions: HashMap<ProjectFile, Subscription>,
    pub(super) editor_order: Vec<ProjectFile>,
    pub(super) open_files: Vec<ProjectFile>,
    pub(super) editor_tabs_scroll: ScrollHandle,
    pub(super) project_tree_scroll: UniformListScrollHandle,
    pub(super) loading_files: HashSet<ProjectFile>,
    pub(super) saving_files: HashSet<ProjectFile>,
    pub(super) persistence_timer: Option<Task<()>>,
    pub(super) load_errors: HashMap<ProjectFile, String>,
    pub(super) notice: Option<String>,
    pub(super) closing: bool,
    pub(super) close_prompt_open: bool,
    pub(super) selected_file: ProjectFile,
    pub(super) expanded_directories: HashSet<PathBuf>,
    pub(super) loading_directories: HashSet<PathBuf>,
    pub(super) moving_path: Option<PathBuf>,
    pub(super) file_operation_running: bool,
    pub(super) project_context_target: Option<(PathBuf, bool, bool)>,
    pub(super) inline_file_edit: Option<InlineFileEdit>,
    pub(super) tree_rows: Vec<TreeRow>,
    pub(super) project_revision: u64,
    pub(super) project_panel_open: bool,
    pub(super) run_panel_open: bool,
    pub(super) project_resize: Entity<ResizableState>,
    pub(super) run_resize: Entity<ResizableState>,
    pub(super) session_ready: bool,
    pub(super) session_revision: u64,
    pub(super) window_view: crate::domain::history::WindowViewState,
    window_diagnostics: std::rc::Rc<std::cell::RefCell<ui::diagnostics::WindowDiagnostics>>,
    window_diagnostics_task: Option<Task<()>>,
    pub(super) window_bounds_subscription: Option<Subscription>,
    pub(super) pending_file_views: HashMap<PathBuf, crate::domain::history::FileViewState>,
}

impl Workspace {
    // Maximum vertical rows per mouse-wheel event in the editor and project tree.
    pub(super) const WHEEL_MAX_LINES: f32 = 3.0;

    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut changes = crate::data_context::DATA_CONTEXT.subscribe_changes();
        cx.spawn_in(window, async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this
                    .update_in(cx, |this, window, cx| {
                        if this.project_revision != DATA_CONTEXT.project.revision() {
                            this.rebuild_tree();
                        }
                        this.sync_run_configuration(window, cx);
                        this.sync_run_views(cx);
                        this.sync_documents(window, cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let session = DATA_CONTEXT.startup().session.clone();
        let notice = DATA_CONTEXT.startup().notice.clone();
        let expanded = session
            .as_ref()
            .map(|saved| saved.expanded_directories.iter().cloned().collect())
            .unwrap_or_else(|| HashSet::from([PathBuf::new(), PathBuf::from("src")]));
        tracing::info!(
            project_width = 300,
            run_height = 250,
            "workspace layout initialized"
        );
        tracing::info!(
            delay_ms = Self::AUTOSAVE_DELAY.as_millis(),
            "workspace and document saving enabled after activity becomes idle"
        );
        let mut this = Self {
            project_windows: HashMap::new(),
            run_configuration_window: None,
            navigation: Vec::new(),
            navigation_index: 0,
            navigating: false,
            search_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("输入要搜索的代码内容…")),
            search_subscription: None,
            search_results: Vec::new(),
            pending_search_positions: HashMap::new(),
            search_loading: false,
            search_request: 0,
            search_error: None,
            layout_subscriptions: Vec::new(),
            git_filter: cx.new(|cx| InputState::new(window, cx).placeholder("Text or hash")),
            git_filter_subscription: None,
            git_console_selected: false,
            selected_git_commit: None,
            git_details_loading: false,
            run_config_revision: DATA_CONTEXT.project.run_configuration().request,
            run_views: HashMap::new(),
            statistics_loading: false,
            statistics_sort: 0,
            statistics_descending: false,
            statistics_selected: None,
            active_tool: session
                .as_ref()
                .map_or(ToolWindow::Run, |saved| saved.active_tool),
            terminal_tabs: vec![TerminalTab {
                id: crate::domain::terminal::TerminalId::Shell,
                label: "Local".into(),
                view: cx.new(|cx| {
                    crate::gui::terminal::TerminalView::new(
                        crate::domain::terminal::TerminalId::Shell,
                        cx,
                    )
                }),
            }],
            active_terminal: 0,
            terminal_sequence: 0,
            run_terminal: cx.new(|cx| {
                crate::gui::terminal::TerminalView::new(
                    crate::domain::terminal::TerminalId::Run,
                    cx,
                )
            }),
            git_loading: false,
            editors: HashMap::new(),
            definition_request: 0,
            definition_observer: None,
            external_source_files: HashSet::new(),
            editor_revisions: HashMap::new(),
            syncing_editors: HashSet::new(),
            editor_subscriptions: HashMap::new(),
            editor_order: Vec::new(),
            open_files: Vec::new(),
            editor_tabs_scroll: ScrollHandle::new(),
            project_tree_scroll: UniformListScrollHandle::new(),
            loading_files: HashSet::new(),
            saving_files: HashSet::new(),
            persistence_timer: None,
            load_errors: HashMap::new(),
            notice,
            closing: false,
            close_prompt_open: false,
            selected_file: ProjectFile("Cargo.toml".into()),
            expanded_directories: expanded,
            loading_directories: HashSet::new(),
            moving_path: None,
            file_operation_running: false,
            project_context_target: None,
            inline_file_edit: None,
            tree_rows: Vec::new(),
            project_revision: 0,
            project_panel_open: session
                .as_ref()
                .is_none_or(|saved| saved.project_panel_open),
            run_panel_open: session.as_ref().is_none_or(|saved| saved.run_panel_open),
            project_resize: cx.new(|_| ResizableState::default()),
            run_resize: cx.new(|_| ResizableState::default()),
            session_ready: false,
            session_revision: session.as_ref().map_or(0, |saved| saved.revision),
            window_view: Self::window_view_state(window),
            window_diagnostics: Default::default(),
            window_diagnostics_task: None,
            window_bounds_subscription: None,
            pending_file_views: session
                .as_ref()
                .map(|saved| {
                    saved
                        .file_views
                        .iter()
                        .cloned()
                        .map(|view| (view.path.clone(), view))
                        .collect()
                })
                .unwrap_or_default(),
        };
        this.start_subscribe(window, cx);
        cx.spawn(async move |_, _| {
            if let Err(error) = crate::application::lifecycle::start_language_server().await {
                tracing::warn!(%error, "Workspace language server startup failed");
            }
        })
        .detach();
        this.window_diagnostics.borrow_mut().bounds_changed(window);
        this.layout_subscriptions
            .push(cx.observe(&this.run_resize, |_, _, cx| cx.notify()));
        this.layout_subscriptions
            .push(cx.observe(&this.project_resize, |_, _, cx| cx.notify()));
        this.git_filter_subscription = Some(cx.observe(&this.git_filter, |_, _, cx| cx.notify()));
        this.search_subscription = Some(cx.subscribe_in(
            &this.search_input,
            window,
            |this, _, event, window, cx| match event {
                gpui_kit::component::input::InputEvent::Change => this.search_project(cx),
                gpui_kit::component::input::InputEvent::PressEnter { .. } => {
                    if !this.search_loading
                        && let Some(result) = this.search_results.first().cloned()
                    {
                        gpui_kit::component::WindowExt::close_dialog(window, cx);
                        this.search_request += 1;
                        this.open_content_match(result, window, cx);
                    }
                }
                _ => {}
            },
        ));
        this.run_views.insert(
            crate::domain::terminal::TerminalId::Run,
            this.run_terminal.clone(),
        );
        this.window_bounds_subscription =
            Some(cx.observe_window_bounds(window, |this, window, cx| {
                this.record_window_bounds(window, cx);
                this.update_window_view(window, cx);
            }));
        this.load_directory(PathBuf::new(), cx);
        let directories: Vec<_> = this
            .expanded_directories
            .iter()
            .filter(|path| !path.as_os_str().is_empty())
            .cloned()
            .collect();
        for directory in directories {
            this.load_directory(directory, cx);
        }
        if let Some(session) = session {
            for path in &session.open_files {
                this.select_file(this.project_file(path), window, cx);
            }
            if let Some(active) = session
                .active_file
                .filter(|active| session.open_files.contains(active))
            {
                this.select_file(this.project_file(&active), window, cx);
            }
            tracing::info!(
                tabs = this.open_files.len(),
                directories = this.expanded_directories.len(),
                "workspace state restored into GUI"
            );
        } else if let Some(path) = DATA_CONTEXT.startup().initial_file.clone() {
            this.select_file(ProjectFile(path), window, cx);
        }
        this.session_ready = true;
        if this.run_panel_open && this.active_tool == ToolWindow::Terminal {
            this.terminal_tabs[0]
                .view
                .update(cx, |terminal, cx| terminal.start(cx));
        }
        if this.run_panel_open && this.active_tool == ToolWindow::Git {
            this.refresh_git(cx);
        }
        if this.run_panel_open && this.active_tool == ToolWindow::Statistics {
            this.refresh_statistics(cx);
        }
        this.schedule_save(cx);
        this.refresh_git(cx);
        this
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = std::time::Instant::now();
        let child = self.render_view(window, cx).into_any_element();
        ui::diagnostics::MeasuredWorkspace::new(
            "workspace",
            child,
            self.window_diagnostics.clone(),
            started,
        )
    }
}
