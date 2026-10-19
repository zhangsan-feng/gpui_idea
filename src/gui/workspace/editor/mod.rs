gpui_kit::actions!(workspace, [SaveDocument, CloseAllDocuments]);
use std::path::Path;
mod core;
mod external;
mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
use gpui_kit::component::input::EditorState;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
pub(in crate::gui::workspace) struct State {
    pub(in crate::gui::workspace) pending_search_positions:
        HashMap<PathBuf, gpui_kit::component::input::Position>,

    pub(in crate::gui::workspace) navigation: Vec<ProjectFile>,
    pub(in crate::gui::workspace) navigation_index: usize,
    pub(in crate::gui::workspace) navigating: bool,
    pub(in crate::gui::workspace) editors: HashMap<ProjectFile, Entity<EditorState>>,
    pub(in crate::gui::workspace) image_previews: HashMap<ProjectFile, ImagePreviewView>,
    pub(in crate::gui::workspace) definition_request: u64,
    pub(in crate::gui::workspace) definition_observer: Option<Subscription>,
    pub(in crate::gui::workspace) external_source_files: HashSet<ProjectFile>,
    pub(in crate::gui::workspace) editor_revisions: HashMap<ProjectFile, u64>,
    pub(in crate::gui::workspace) syncing_editors: HashSet<ProjectFile>,
    pub(in crate::gui::workspace) editor_subscriptions: HashMap<ProjectFile, Subscription>,
    pub(in crate::gui::workspace) editor_order: Vec<ProjectFile>,
    pub(in crate::gui::workspace) open_files: Vec<ProjectFile>,
    pub(in crate::gui::workspace) editor_tabs_scroll: ScrollHandle,
    pub(in crate::gui::workspace) loading_files: HashSet<ProjectFile>,
    pub(in crate::gui::workspace) saving_files: HashSet<ProjectFile>,
    pub(in crate::gui::workspace) load_errors: HashMap<ProjectFile, String>,
    pub(in crate::gui::workspace) selected_file: ProjectFile,
    pub(in crate::gui::workspace) pending_file_views:
        HashMap<PathBuf, crate::domain::history::FileViewState>,
}

pub(crate) struct Editor {
    pub(in crate::gui::workspace) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui::workspace) fn init_component_data(
    _window: &mut Window,
    _cx: &mut App,
    session: &Option<crate::domain::history::WorkspaceSession>,
) -> State {
    State {
        pending_search_positions: HashMap::new(),

        navigation: Vec::new(),
        navigation_index: 0,
        navigating: false,
        editors: HashMap::new(),
        image_previews: HashMap::new(),
        definition_request: 0,
        definition_observer: None,
        external_source_files: HashSet::new(),
        editor_revisions: HashMap::new(),
        syncing_editors: HashSet::new(),
        editor_subscriptions: HashMap::new(),
        editor_order: Vec::new(),
        open_files: Vec::new(),
        editor_tabs_scroll: ScrollHandle::new(),
        loading_files: HashSet::new(),
        saving_files: HashSet::new(),
        load_errors: HashMap::new(),
        selected_file: ProjectFile("Cargo.toml".into()),
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
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProjectFile(pub PathBuf);
impl ProjectFile {
    pub(in crate::gui::workspace) fn name(&self) -> String {
        self.0
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
    pub(in crate::gui::workspace) fn relative_path(&self) -> &std::path::Path {
        &self.0
    }
    pub(in crate::gui::workspace) fn path(&self, root: &std::path::Path) -> PathBuf {
        root.join(&self.0)
    }
}

#[derive(Clone)]
pub(in crate::gui) struct DraggedEditorTab {
    pub(in crate::gui::workspace) external_source: bool,
    pub(in crate::gui::workspace) file: ProjectFile,
    pub(in crate::gui::workspace) label: String,
}

pub(in crate::gui) struct ImagePreviewView {
    pub(in crate::gui::workspace) image: Option<std::sync::Arc<gpui_kit::Image>>,
    pub(in crate::gui::workspace) revision: u64,
    pub(in crate::gui::workspace) bytes: usize,
    pub(in crate::gui::workspace) error: Option<String>,
}

impl ProjectFile {
    pub(in crate::gui::workspace) fn for_path(path: &Path) -> ProjectFile {
        ProjectFile(
            path.strip_prefix(DATA_CONTEXT.project_root())
                .unwrap_or(path)
                .to_owned(),
        )
    }
}

#[derive(Clone)]
pub(crate) enum Event {
    SelectFile(ProjectFile),
    SaveCurrent,
    CloseAllEditorTabs,
    Navigate(bool),
    OpenContentMatch(crate::domain::project::ContentMatch),
    OpenDroppedFiles(gpui_kit::ExternalPaths),
    Restore,
    RemapFiles(std::path::PathBuf, std::path::PathBuf),
    RemoveFiles(std::path::PathBuf),
}

impl Editor {
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
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::Editor);
        let mut events = crate::event_bus::EVENT_BUS.gui_event.subscribe(
            self.workspace.entity_id(),
            crate::event_bus::refresh_gui::GuiComponent::Editor,
        );
        self.subscription = Some(cx.spawn_in(window, async move |view,cx| {
            loop {
                let event = tokio::select! {
                    changed = refresh.changed() => { if changed.is_err() { break; } None }
                    event = events.receiver.recv() => { let Some(event) = event else { break; }; Some(event) }
                };
                if view.update_in(cx,|this,window,cx| {
                    if let Some(event) = event { this.handle_event(event,window,cx); }
                    else { this.sync_documents(window,cx); }
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
        if let crate::event_bus::gui_event::GuiEvent::Editor(event) = event {
            match event {
                Event::SelectFile(value) => {
                    self.select_file(value, window, cx);
                }
                Event::SaveCurrent => {
                    self.save_current(window, cx);
                }
                Event::CloseAllEditorTabs => {
                    self.close_all_editor_tabs(cx);
                }
                Event::Navigate(value) => {
                    self.navigate(value, window, cx);
                }
                Event::OpenContentMatch(value) => {
                    self.open_content_match(value, window, cx);
                }
                Event::OpenDroppedFiles(value) => {
                    self.open_dropped_files(&value, window, cx);
                }
                Event::Restore => {
                    self.restore(window, cx);
                }
                Event::RemapFiles(value0, value1) => {
                    self.remap_files(&value0, &value1, window, cx);
                }
                Event::RemoveFiles(value) => {
                    self.remove_files(&value, window, cx);
                }
            }
        }
    }
}
impl Render for Editor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_editor(cx)
    }
}
