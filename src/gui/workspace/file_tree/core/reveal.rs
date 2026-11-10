use crate::{
    data_context::DATA_CONTEXT,
    event_bus::gui_event::GuiEvent,
    gui::workspace::{ProjectFile, file_tree::FileTree, left_sidebar, window_bar},
};
use gpui_kit::{Context, ScrollStrategy, Window};
use std::path::Path;

impl FileTree {
    pub(in crate::gui::workspace::file_tree) fn reveal_file(
        &mut self,
        file: ProjectFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !DATA_CONTEXT.has_project() {
            return;
        }
        let absolute = file.path(DATA_CONTEXT.project_root());
        let Ok(path) = absolute.strip_prefix(DATA_CONTEXT.project_root()) else {
            return;
        };
        let path = path.to_owned();
        tracing::info!(file = %path.display(), "revealing editor file in project tree");
        self.send(GuiEvent::LeftSidebar(left_sidebar::Event::OpenProjectPanel));
        self.state.pending_reveal = Some(path.clone());
        self.state.project_selection.clear();
        self.state.project_selection.insert(path.clone());
        let (_, snapshots) = DATA_CONTEXT.project.file_tree.snapshots();
        for directory in path.parent().unwrap_or(Path::new("")).ancestors() {
            self.state.expanded_directories.insert(directory.to_owned());
            if !snapshots
                .get(directory)
                .is_some_and(|snapshot| snapshot.error.is_none())
            {
                self.load_directory(directory.to_owned(), cx);
            }
        }
        self.rebuild_tree();
        self.state.project_tree_focus.focus(window, cx);
        self.send(GuiEvent::WindowBar(window_bar::Event::ScheduleSave));
        cx.notify();
    }

    pub(super) fn scroll_to_revealed_file(&mut self) {
        let Some(path) = self.state.pending_reveal.as_ref() else {
            return;
        };
        let Some(index) = self.state.tree_rows.iter().position(|row| {
            matches!(row, super::super::TreeRow::Entry(entry, _) if &entry.relative_path == path)
        }) else {
            return;
        };
        self.state.project_selection.clear();
        self.state.project_selection.insert(path.clone());
        self.state
            .project_tree_scroll
            .scroll_to_item(index, ScrollStrategy::Top);
        self.state.pending_reveal = None;
    }
}
