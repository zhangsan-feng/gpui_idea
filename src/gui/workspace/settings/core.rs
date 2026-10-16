use super::*;

impl WorkspaceSettings {
    pub(super) fn select_section(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < SECTIONS.len() && self.selected_section != index {
            self.selected_section = index;
            cx.notify();
        }
    }
}
