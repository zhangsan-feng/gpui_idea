use super::super::*;

impl Editor {
    pub(in crate::gui::main_window::content_area::editor) fn can_reposition_tab(
        &self,
        file: &ProjectFile,
        cx: &App,
    ) -> bool {
        self.state.open_files.contains(file) && self.can_select_file(file, cx)
    }
    pub(in crate::gui::main_window::content_area::editor) fn side_for(
        &self,
        file: &ProjectFile,
    ) -> EditorSide {
        if self
            .state
            .split
            .as_ref()
            .is_some_and(|split| split.right_files.contains(file))
        {
            EditorSide::Right
        } else {
            EditorSide::Left
        }
    }

    pub(in crate::gui::main_window::content_area::editor) fn pane_focus(
        &self,
        side: EditorSide,
    ) -> &FocusHandle {
        match (&self.state.split, side) {
            (Some(split), EditorSide::Right) => &split.right_pane_focus,
            _ => &self.state.left_pane_focus,
        }
    }

    pub(in crate::gui::main_window::content_area::editor) fn focus_file(
        &self,
        file: &ProjectFile,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.markdown_mode(file) != Some(MarkdownMode::Rendered) {
            if let Some(editor) = self.state.editors.get(file) {
                editor.update(cx, |state, cx| state.focus(window, cx));
                return;
            }
        }
        self.pane_focus(self.side_for(file)).focus(window, cx);
    }

    pub(in crate::gui::main_window::content_area::editor) fn selected_in(
        &self,
        side: EditorSide,
    ) -> Option<&ProjectFile> {
        match (&self.state.split, side) {
            (Some(split), EditorSide::Left) => split.left_selected.as_ref(),
            (Some(split), EditorSide::Right) => split.right_selected.as_ref(),
            (None, EditorSide::Left) if !self.state.open_files.is_empty() => {
                Some(&self.state.selected_file)
            }
            _ => None,
        }
    }

    pub(in crate::gui::main_window::content_area::editor) fn tab_scroll(
        &self,
        side: EditorSide,
    ) -> &ScrollHandle {
        match (&self.state.split, side) {
            (Some(split), EditorSide::Right) => &split.right_scroll,
            _ => &self.state.editor_tabs_scroll,
        }
    }

    pub(super) fn assign_editor_side(&mut self, file: &ProjectFile, side: EditorSide) {
        if let Some(split) = &mut self.state.split {
            match side {
                EditorSide::Left => {
                    split.right_files.remove(file);
                }
                EditorSide::Right => {
                    split.right_files.insert(file.clone());
                }
            }
        }
    }

    pub(super) fn remember_pane_selection(&mut self, file: &ProjectFile) {
        let side = self.side_for(file);
        if let Some(split) = &mut self.state.split {
            match side {
                EditorSide::Left => split.left_selected = Some(file.clone()),
                EditorSide::Right => split.right_selected = Some(file.clone()),
            }
        }
    }

    pub(super) fn repair_split(&mut self, collapse_empty: bool) {
        let Some(split) = &mut self.state.split else {
            return;
        };
        let files = &self.state.open_files;
        split.right_files.retain(|file| files.contains(file));
        let left = |file: &ProjectFile| !split.right_files.contains(file);
        let right = |file: &ProjectFile| split.right_files.contains(file);
        if !split
            .left_selected
            .as_ref()
            .is_some_and(|file| files.contains(file) && left(file))
        {
            split.left_selected = files.iter().find(|file| left(file)).cloned();
        }
        if !split
            .right_selected
            .as_ref()
            .is_some_and(|file| files.contains(file) && right(file))
        {
            split.right_selected = files.iter().find(|file| right(file)).cloned();
        }
        if collapse_empty && (split.left_selected.is_none() || split.right_selected.is_none()) {
            self.state.split = None;
        }
    }

    pub(in crate::gui::main_window::content_area::editor) fn open_tab_on_side(
        &mut self,
        file: &ProjectFile,
        side: EditorSide,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.can_reposition_tab(file, cx) {
            return;
        }
        if self.state.split.is_none() {
            let right_files = match side {
                EditorSide::Left => self
                    .state
                    .open_files
                    .iter()
                    .filter(|other| *other != file)
                    .cloned()
                    .collect(),
                EditorSide::Right => HashSet::from([file.clone()]),
            };
            let previous = Some(self.state.selected_file.clone());
            self.state.split = Some(SplitEditor {
                right_files,
                left_selected: previous.clone(),
                right_selected: previous,
                right_scroll: ScrollHandle::new(),
                right_pane_focus: cx.focus_handle(),
                right_markdown_resize: cx.new(|_| gpui_kit::component::ResizableState::default()),
                resize: cx.new(|_| gpui_kit::component::ResizableState::default()),
            });
        } else {
            self.assign_editor_side(file, side);
        }
        self.repair_split(false);
        self.select_file(file.clone(), window, cx);
    }

    pub(in crate::gui::main_window::content_area::editor) fn close_empty_pane(
        &mut self,
        side: EditorSide,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_in(side).is_none() {
            self.state.split = None;
            self.reveal_editor_tab(&self.state.selected_file);
            self.focus_file(&self.state.selected_file, window, cx);
            cx.notify();
        }
    }

    pub(in crate::gui::main_window::content_area::editor) fn activate_visible_file(
        &mut self,
        file: &ProjectFile,
        cx: &mut Context<Self>,
    ) {
        if self.state.selected_file == *file
            || self.selected_in(self.side_for(file)) != Some(file)
            || !self.can_select_file(file, cx)
        {
            return;
        }
        self.record_file_activation(file, cx);
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomSidebar,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
        ]);
    }
}
