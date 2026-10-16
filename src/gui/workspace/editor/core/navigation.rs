use super::super::*;
use crate::gui::workspace::editor::Editor;
impl Editor {
    pub(in crate::gui::workspace) fn navigate(
        &mut self,
        forward: bool,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) {
        let index = if forward {
            self.state
                .navigation_index
                .checked_add(1)
                .filter(|index| *index < self.state.navigation.len())
        } else {
            self.state.navigation_index.checked_sub(1)
        };
        let Some(index) = index else {
            return;
        };
        let file = self.state.navigation[index].clone();
        if self
            .peer_file_tree(cx)
            .read(cx)
            .state
            .moving_path
            .as_ref()
            .is_some_and(|source| file.0.starts_with(source))
        {
            return;
        }
        self.state.navigation_index = index;
        self.state.navigating = true;
        self.select_file(file, window, cx);
        self.state.navigating = false;
    }
}
