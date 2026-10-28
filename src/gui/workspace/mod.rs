pub(crate) mod bottom_bar;
pub(crate) mod bottom_panel;
pub(crate) mod editor;
pub(crate) mod file_tree;
pub(crate) mod left_sidebar;
pub(crate) mod right_sidebar;
pub(crate) mod window_bar;

mod ui;

use gpui_kit::{AppContext as _, Context, IntoElement, Render, Window};

pub(super) use editor::{CloseAllDocuments, SaveDocument};

pub(super) use crate::domain::history::ToolWindow;
pub(super) use bottom_panel::TerminalTab;
pub(super) use editor::{ImagePreviewView, ProjectFile};
pub(super) use file_tree::{DraggedProjectEntry, FileTreeMenuAction, InlineFileEdit, TreeRow};
pub(super) use window_bar::ToolbarAction;

pub struct Workspace {
    pub(in crate::gui) editor: gpui_kit::Entity<editor::Editor>,
    pub(in crate::gui) file_tree: gpui_kit::Entity<file_tree::FileTree>,
    pub(in crate::gui) window_bar: gpui_kit::Entity<window_bar::WindowBar>,
    pub(in crate::gui) bottom_bar: gpui_kit::Entity<bottom_bar::BottomBar>,
    pub(in crate::gui) bottom_panel: gpui_kit::Entity<bottom_panel::BottomPanel>,
    pub(in crate::gui) left_sidebar: gpui_kit::Entity<left_sidebar::LeftSidebar>,
    pub(in crate::gui) right_sidebar: gpui_kit::Entity<right_sidebar::RightSidebar>,
}
impl Workspace {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let parent = cx.weak_entity();
        let this = Self {
            editor: cx.new(|cx| editor::Editor::new(parent.clone(), window, cx)),
            file_tree: cx.new(|cx| file_tree::FileTree::new(parent.clone(), window, cx)),
            window_bar: cx.new(|cx| window_bar::WindowBar::new(parent.clone(), window, cx)),
            bottom_bar: cx.new(|cx| bottom_bar::BottomBar::new(parent.clone(), window, cx)),
            bottom_panel: cx.new(|cx| bottom_panel::BottomPanel::new(parent.clone(), window, cx)),
            left_sidebar: cx.new(|cx| left_sidebar::LeftSidebar::new(parent.clone(), window, cx)),
            right_sidebar: cx
                .new(|cx| right_sidebar::RightSidebar::new(parent.clone(), window, cx)),
        };
        crate::event_bus::EVENT_BUS.gui_event.send(
            cx.entity().entity_id(),
            crate::event_bus::gui_event::GuiEvent::WindowBar(window_bar::Event::Initialize),
        );
        this
    }
}
impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let started = std::time::Instant::now();
        let diagnostics = self.window_bar.read(cx).state.window_diagnostics.clone();
        let child = self.render_view(window, cx).into_any_element();
        crate::gui::component::measured_element::MeasuredElement::new(
            child,
            started,
            move |times, window, cx| {
                diagnostics
                    .borrow_mut()
                    .record("workspace", times, window, cx)
            },
        )
    }
}
