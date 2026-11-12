gpui_kit::actions!(
    workspace,
    [
        FormatCode,
        SaveDocument,
        CloseAllDocuments,
        RunEditorFile,
        RevealEditorFile,
        CopyEditorProjectPath,
        CopyEditorAbsolutePath
    ]
);
use crate::domain::history::MarkdownMode;
use std::path::Path;
mod core;
mod external;
mod ui;
use crate::gui::main_window::Workspace;
use crate::gui::main_window::ui::*;
use gpui_kit::component::input::EditorState;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
pub(in crate::gui) struct State {
    pub(in crate::gui) soft_wrap: bool,
    pub(in crate::gui) markdown_resize: Entity<gpui_kit::component::ResizableState>,
    pub(in crate::gui) markdown_extensions:
        HashMap<ProjectFile, gpui_kit::base::text::MarkdownExtensions>,
    pub(in crate::gui) markdown_modes: HashMap<ProjectFile, MarkdownMode>,
    pub(in crate::gui) pending_search_positions:
        HashMap<PathBuf, gpui_kit::component::input::Position>,

    pub(in crate::gui) navigation: Vec<ProjectFile>,
    pub(in crate::gui) navigation_index: usize,
    pub(in crate::gui) navigating: bool,
    pub(in crate::gui) editors: HashMap<ProjectFile, Entity<EditorState>>,
    pub(in crate::gui) image_previews: HashMap<ProjectFile, ImagePreviewView>,
    pub(in crate::gui) definition_request: u64,
    pub(in crate::gui) definition_observer: Option<Subscription>,
    pub(in crate::gui) external_source_files: HashSet<ProjectFile>,
    pub(in crate::gui) editor_revisions: HashMap<ProjectFile, u64>,
    pub(in crate::gui) syncing_editors: HashSet<ProjectFile>,
    pub(in crate::gui) editor_subscriptions: HashMap<ProjectFile, Subscription>,
    pub(in crate::gui) editor_order: Vec<ProjectFile>,
    pub(in crate::gui) open_files: Vec<ProjectFile>,
    pub(in crate::gui) editor_tabs_scroll: ScrollHandle,
    pub(in crate::gui) loading_files: HashSet<ProjectFile>,
    pub(in crate::gui) formatting_files: HashSet<ProjectFile>,
    pub(in crate::gui) saving_files: HashSet<ProjectFile>,
    pub(in crate::gui) load_errors: HashMap<ProjectFile, String>,
    pub(in crate::gui) selected_file: ProjectFile,
    pub(in crate::gui) pending_file_views: HashMap<PathBuf, crate::domain::history::FileViewState>,
}

pub(crate) struct Editor {
    pub(in crate::gui) state: State,
    workspace: gpui_kit::WeakEntity<Workspace>,
    subscription: Option<gpui_kit::Task<()>>,
}

pub(in crate::gui) fn init_component_data(
    _window: &mut Window,
    cx: &mut App,
    session: &Option<crate::domain::history::WorkspaceSession>,
) -> State {
    State {
        soft_wrap: DATA_CONTEXT.settings.snapshot().general.soft_wrap,
        markdown_modes: session
            .as_ref()
            .map(|saved| {
                saved
                    .file_views
                    .iter()
                    .filter(|view| {
                        crate::gui::component::code_editor::language_for_path(&view.path)
                            == "markdown"
                    })
                    .map(|view| (ProjectFile::for_path(&view.path), view.markdown_mode))
                    .collect()
            })
            .unwrap_or_default(),
        markdown_extensions: HashMap::new(),
        markdown_resize: cx.new(|_| gpui_kit::component::ResizableState::default()),
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
        formatting_files: HashSet::new(),
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

pub(super) struct MarkdownImages {
    document: PathBuf,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ProjectFile(pub PathBuf);
impl ProjectFile {
    pub(in crate::gui) fn name(&self) -> String {
        self.0
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
    pub(in crate::gui) fn relative_path(&self) -> &std::path::Path {
        &self.0
    }
    pub(in crate::gui) fn path(&self, root: &std::path::Path) -> PathBuf {
        root.join(&self.0)
    }
}

#[derive(Clone)]
pub(in crate::gui) struct DraggedEditorTab {
    pub(in crate::gui) external_source: bool,
    pub(in crate::gui) file: ProjectFile,
    pub(in crate::gui) label: String,
}

pub(in crate::gui) struct ImagePreviewView {
    pub(in crate::gui) zoom: f32,
    pub(in crate::gui) scroll: ScrollHandle,
    pub(in crate::gui) image: Option<std::sync::Arc<gpui_kit::Image>>,
    pub(in crate::gui) revision: u64,
    pub(in crate::gui) bytes: usize,
    pub(in crate::gui) error: Option<String>,
}

impl ProjectFile {
    pub(in crate::gui) fn for_path(path: &Path) -> ProjectFile {
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
    FormatCurrent,
    CloseAllEditorTabs,
    Navigate(bool),
    OpenContentMatch(crate::domain::project::ContentMatch),
    OpenDroppedFiles(gpui_kit::ExternalPaths),
    Restore,
    RemapFiles(std::path::PathBuf, std::path::PathBuf),
    RemoveFiles(std::path::PathBuf),
}

impl Editor {
    pub(in crate::gui::main_window) fn new(
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
                            this.sync_documents(window, cx);
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
        if let crate::event_bus::gui_event::GuiEvent::Editor(event) = event {
            match event {
                Event::SelectFile(value) => {
                    self.select_file(value, window, cx);
                }
                Event::FormatCurrent => self.format_current(window, cx),
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_editor(window, cx)
    }
}
