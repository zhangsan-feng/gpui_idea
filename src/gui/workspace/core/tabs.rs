use super::super::{ProjectFile, Workspace};
use gpui_kit::{Context, Window};

impl Workspace {
    pub(in crate::gui::workspace) fn close_all_editor_tabs(&mut self, cx: &mut Context<Self>) {
        if self.closing || self.close_prompt_open || self.open_files.is_empty() {
            return;
        }
        let closed = self.open_files.len();
        self.open_files.clear();
        // As with closing one tab, dirty buffers and pending autosave stay alive.
        self.prune_editors();
        self.schedule_save(cx);
        tracing::info!(closed, "all editor tabs closed");
        cx.notify();
    }

    pub(in crate::gui::workspace) fn reveal_editor_tab(&self, file: &ProjectFile) {
        if let Some(index) = self.open_files.iter().position(|opened| opened == file) {
            self.editor_tabs_scroll.scroll_to_item(index);
        }
    }

    pub(in crate::gui::workspace) fn close_editor_tab(
        &mut self,
        file: &ProjectFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open {
            return;
        }
        let Some(index) = self.open_files.iter().position(|opened| opened == file) else {
            return;
        };
        self.open_files.remove(index);
        // Retain the buffer, undo history and pending autosave when a tab closes.
        if self.selected_file == *file && !self.open_files.is_empty() {
            let next = self.open_files[index.min(self.open_files.len() - 1)].clone();
            self.select_file(next, window, cx);
        }
        self.prune_editors();
        self.schedule_save(cx);
        tracing::info!(file = %file.relative_path().display(), remaining = self.open_files.len(), "editor tab closed");
        cx.notify();
    }

    pub(in crate::gui::workspace) fn move_editor_tab(
        &mut self,
        file: &ProjectFile,
        target: &ProjectFile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.closing || self.close_prompt_open || file == target {
            return;
        }
        let Some(from) = self.open_files.iter().position(|opened| opened == file) else {
            return;
        };
        let Some(target_index) = self.open_files.iter().position(|opened| opened == target) else {
            return;
        };
        let after = self
            .editor_tabs_scroll
            .bounds_for_item(target_index)
            .is_some_and(|bounds| {
                window.mouse_position().x > bounds.center().x + self.editor_tabs_scroll.offset().x
            });
        let insertion = target_index + usize::from(after);
        let to = insertion - usize::from(from < insertion);
        let moved = self.open_files.remove(from);
        self.open_files.insert(to, moved);
        tracing::info!(file = %file.relative_path().display(), from, to, "editor tab reordered");
        self.select_file(file.clone(), window, cx);
    }

    pub(in crate::gui::workspace) fn editor_tab_label(&self, file: &ProjectFile) -> String {
        let name = file.name();
        if self
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
