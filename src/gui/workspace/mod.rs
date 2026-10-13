use crate::data_context::DATA_CONTEXT;
mod core;
mod external;
mod ui;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use gpui_kit::component::ResizableState;
use gpui_kit::component::input::EditorState;
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, Render, ScrollHandle, Subscription, Task,
    UniformListScrollHandle, Window,
};

gpui_kit::actions!(workspace, [SaveDocument, CloseAllDocuments]);

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
    Message {
        directory: PathBuf,
        depth: usize,
        text: String,
        retry: bool,
    },
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

pub struct Workspace {
    pub(super) editors: HashMap<ProjectFile, Entity<EditorState>>,
    pub(super) editor_subscriptions: HashMap<ProjectFile, Subscription>,
    pub(super) editor_order: Vec<ProjectFile>,
    pub(super) open_files: Vec<ProjectFile>,
    pub(super) editor_tabs_scroll: ScrollHandle,
    pub(super) project_tree_scroll: UniformListScrollHandle,
    pub(super) loading_files: HashSet<ProjectFile>,
    pub(super) saving_files: HashSet<ProjectFile>,
    pub(super) autosave_timers: HashMap<ProjectFile, Task<()>>,
    pub(super) load_errors: HashMap<ProjectFile, String>,
    pub(super) notice: Option<String>,
    pub(super) closing: bool,
    pub(super) close_prompt_open: bool,
    pub(super) selected_file: ProjectFile,
    pub(super) expanded_directories: HashSet<PathBuf>,
    pub(super) loading_directories: HashSet<PathBuf>,
    pub(super) moving_path: Option<PathBuf>,
    pub(super) tree_rows: Vec<TreeRow>,
    pub(super) project_revision: u64,
    pub(super) project_panel_open: bool,
    pub(super) run_panel_open: bool,
    pub(super) project_resize: Entity<ResizableState>,
    pub(super) run_resize: Entity<ResizableState>,
    pub(super) session_ready: bool,
    pub(super) session_revision: u64,
    pub(super) session_timer: Option<Task<()>>,
    pub(super) window_view: crate::domain::history::WindowViewState,
    pub(super) window_bounds_subscription: Option<Subscription>,
    pub(super) pending_file_views: HashMap<PathBuf, crate::domain::history::FileViewState>,
}

impl Workspace {
    // Maximum vertical rows per mouse-wheel event in the editor and project tree.
    pub(super) const WHEEL_MAX_LINES: f32 = 3.0;

    fn start_subscribe(&mut self, cx: &mut Context<Self>) {
        let mut changes = DATA_CONTEXT.subscribe_gui_refresh();
        cx.spawn(async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this
                    .update(cx, |this, cx| {
                        if this.project_revision != DATA_CONTEXT.project.revision() {
                            this.rebuild_tree();
                        }
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
            "automatic saving enabled after editing becomes idle"
        );
        let mut this = Self {
            editors: HashMap::new(),
            editor_subscriptions: HashMap::new(),
            editor_order: Vec::new(),
            open_files: Vec::new(),
            editor_tabs_scroll: ScrollHandle::new(),
            project_tree_scroll: UniformListScrollHandle::new(),
            loading_files: HashSet::new(),
            saving_files: HashSet::new(),
            autosave_timers: HashMap::new(),
            load_errors: HashMap::new(),
            notice,
            closing: false,
            close_prompt_open: false,
            selected_file: ProjectFile("Cargo.toml".into()),
            expanded_directories: expanded,
            loading_directories: HashSet::new(),
            moving_path: None,
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
            session_timer: None,
            window_view: Self::window_view_state(window),
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
        this.start_subscribe(cx);
        this.window_bounds_subscription =
            Some(cx.observe_window_bounds(window, |this, window, cx| {
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
        } else {
            this.select_file(ProjectFile("Cargo.toml".into()), window, cx);
        }
        this.session_ready = true;
        this.schedule_session_save(cx);
        this
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_view(window, cx)
    }
}
