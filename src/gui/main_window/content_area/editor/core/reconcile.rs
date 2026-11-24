use super::super::*;
use crate::application::APPLICATION;
use std::path::Path;
impl Editor {
    pub(in crate::gui) fn remap_files(
        &mut self,
        source: &Path,
        destination: &Path,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        let remap = |path: &Path| {
            path.strip_prefix(source)
                .map(|suffix| destination.join(suffix))
                .unwrap_or_else(|_| path.to_owned())
        };
        self.state
            .markdown_extensions
            .retain(|file, _| !file.relative_path().starts_with(source));
        let root = DATA_CONTEXT.project_root().to_owned();
        let reopened: Vec<_> = self
            .state
            .open_files
            .iter()
            .filter(|file| file.relative_path().starts_with(source))
            .map(|file| ProjectFile(remap(file.relative_path())))
            .collect();
        let old_selected = self.state.selected_file.clone();
        self.state.markdown_modes = std::mem::take(&mut self.state.markdown_modes)
            .into_iter()
            .map(|(file, mode)| (ProjectFile(remap(file.relative_path())), mode))
            .collect();
        for file in self.state.open_files.clone() {
            if file.relative_path().starts_with(source) {
                self.release_image_tab(&file);
            }
        }
        for file in self
            .state
            .editor_order
            .iter()
            .filter(|file| file.relative_path().starts_with(source))
        {
            let old_path = file.path(&root);
            if let Some(editor) = self.state.editors.get(file) {
                let state = editor.read(cx);
                let cursor = state.cursor_position();
                let scroll = state.scroll_offset();
                let new_path = root.join(remap(file.relative_path()));
                self.state.pending_file_views.insert(
                    new_path.clone(),
                    crate::domain::history::FileViewState {
                        markdown_mode: self
                            .state
                            .markdown_modes
                            .get(&ProjectFile(remap(file.relative_path())))
                            .copied()
                            .unwrap_or_default(),
                        path: new_path,
                        cursor_line: cursor.line,
                        cursor_character: cursor.character,
                        scroll_x: scroll.x.into(),
                        scroll_y: scroll.y.into(),
                    },
                );
            }
            APPLICATION.documents.release_clean(&old_path);
        }
        self.state
            .editors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .editor_subscriptions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .load_errors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .editor_order
            .retain(|file| !file.relative_path().starts_with(source));
        self.state
            .editor_revisions
            .retain(|file, _| !file.relative_path().starts_with(source));
        for file in &mut self.state.open_files {
            file.0 = remap(&file.0);
        }
        let selections = self.state.split.as_mut().map(|split| {
            split.right_files = std::mem::take(&mut split.right_files)
                .into_iter()
                .map(|file| ProjectFile(remap(file.relative_path())))
                .collect();
            for file in [&mut split.left_selected, &mut split.right_selected]
                .into_iter()
                .flatten()
            {
                file.0 = remap(&file.0);
            }
            (split.left_selected.clone(), split.right_selected.clone())
        });
        for file in &mut self.state.navigation {
            file.0 = remap(&file.0);
        }
        let new_selected = ProjectFile(remap(old_selected.relative_path()));
        self.state.selected_file = new_selected.clone();
        let navigating = self.state.navigating;
        self.state.navigating = true;
        for file in reopened {
            self.select_file(file, window, cx);
        }
        if let (Some(split), Some((left, right))) = (&mut self.state.split, selections) {
            split.left_selected = left;
            split.right_selected = right;
        }
        self.repair_split(false);
        if self.state.open_files.contains(&new_selected) {
            self.select_file(new_selected, window, cx);
        }
        self.state.navigating = navigating;
        // Application has already published directory snapshots for this move.
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::main_window::window_bar::Event::ScheduleSave,
        ));
        tracing::info!(source = %source.display(), destination = %destination.display(), reopened_tabs = self.state.open_files.len(), "project tree updated after move");
    }
}

impl Editor {
    pub(in crate::gui) fn remove_files(
        &mut self,
        source: &Path,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        self.state
            .markdown_extensions
            .retain(|file, _| !file.relative_path().starts_with(source));
        let selected_removed = self.state.selected_file.relative_path().starts_with(source);
        let side = self.side_for(&self.state.selected_file);
        self.state
            .markdown_modes
            .retain(|file, _| !file.relative_path().starts_with(source));
        for file in self.state.open_files.clone() {
            if file.relative_path().starts_with(source) {
                self.release_image_tab(&file);
            }
        }

        self.state
            .open_files
            .retain(|file| !file.relative_path().starts_with(source));
        let was_split = self.state.split.is_some();
        self.repair_split(true);
        self.state
            .editors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .editor_subscriptions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .editor_revisions
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .editor_order
            .retain(|file| !file.relative_path().starts_with(source));
        self.state
            .load_errors
            .retain(|file, _| !file.relative_path().starts_with(source));
        self.state
            .pending_file_views
            .retain(|path, _| !path.starts_with(DATA_CONTEXT.project_root().join(source)));
        self.state
            .navigation
            .retain(|file| !file.relative_path().starts_with(source));
        self.state.navigation_index = self
            .state
            .navigation_index
            .min(self.state.navigation.len().saturating_sub(1));

        if selected_removed {
            if let Some(file) = self
                .selected_in(side)
                .filter(|file| !file.relative_path().starts_with(source))
                .or_else(|| self.state.open_files.first())
                .cloned()
            {
                self.select_file(file, window, cx);
            }
        } else if was_split && self.state.split.is_none() && !self.state.open_files.is_empty() {
            self.reveal_editor_tab(&self.state.selected_file);
            self.focus_file(&self.state.selected_file, window, cx);
        }
    }
}
