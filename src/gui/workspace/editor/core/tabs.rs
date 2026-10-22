use crate::gui::workspace::ProjectFile;
use crate::gui::workspace::editor::Editor;
use gpui_kit::{Context, Window};

impl Editor {
    pub(in crate::gui::workspace) fn close_all_editor_tabs(&mut self, cx: &mut Context<Editor>) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || self.state.open_files.is_empty()
        {
            return;
        }
        let closed = self.state.open_files.len();
        for file in self.state.open_files.clone() {
            self.release_image_tab(&file);
        }
        self.state.open_files.clear();
        // Dirty buffers stay alive until autosave completes; clean buffers release now.
        self.prune_editors();
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        tracing::info!(closed, "all editor tabs closed");
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
    }

    pub(in crate::gui::workspace) fn reveal_editor_tab(&self, file: &ProjectFile) {
        if let Some(index) = self
            .state
            .open_files
            .iter()
            .position(|opened| opened == file)
        {
            self.state.editor_tabs_scroll.scroll_to_item(index);
        }
    }

    pub(in crate::gui::workspace) fn close_editor_tab(
        &mut self,
        file: &ProjectFile,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
        {
            return;
        }
        let Some(index) = self
            .state
            .open_files
            .iter()
            .position(|opened| opened == file)
        else {
            return;
        };
        self.state.open_files.remove(index);
        self.release_image_tab(file);
        // Keep unsaved buffers until autosave finishes; release clean closed editors.
        if self.state.selected_file == *file && !self.state.open_files.is_empty() {
            let next = self.state.open_files[index.min(self.state.open_files.len() - 1)].clone();
            self.select_file(next, window, cx);
        }
        self.prune_editors();
        self.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
            crate::gui::workspace::window_bar::Event::ScheduleSave,
        ));
        tracing::info!(file = %file.relative_path().display(), remaining = self.state.open_files.len(), "editor tab closed");
        crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
            crate::event_bus::refresh_gui::GuiComponent::Editor,
            crate::event_bus::refresh_gui::GuiComponent::WindowBar,
            crate::event_bus::refresh_gui::GuiComponent::FileTree,
            crate::event_bus::refresh_gui::GuiComponent::BottomBar,
        ]);
    }

    pub(in crate::gui::workspace) fn move_editor_tab(
        &mut self,
        file: &ProjectFile,
        target: &ProjectFile,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
            || file == target
        {
            return;
        }
        let Some(from) = self
            .state
            .open_files
            .iter()
            .position(|opened| opened == file)
        else {
            return;
        };
        let Some(target_index) = self
            .state
            .open_files
            .iter()
            .position(|opened| opened == target)
        else {
            return;
        };
        let after = self
            .state
            .editor_tabs_scroll
            .bounds_for_item(target_index)
            .is_some_and(|bounds| {
                window.mouse_position().x
                    > bounds.center().x + self.state.editor_tabs_scroll.offset().x
            });
        let insertion = target_index + usize::from(after);
        let to = insertion - usize::from(from < insertion);
        let moved = self.state.open_files.remove(from);
        self.state.open_files.insert(to, moved);
        tracing::info!(file = %file.relative_path().display(), from, to, "editor tab reordered");
        self.select_file(file.clone(), window, cx);
    }

    pub(in crate::gui::workspace) fn editor_tab_label(&self, file: &ProjectFile) -> String {
        let name = file.name();
        if self
            .state
            .open_files
            .iter()
            .any(|other| other != file && other.name() == name)
        {
            file.relative_path().to_string_lossy().into_owned()
        } else {
            name
        }
    }
}
