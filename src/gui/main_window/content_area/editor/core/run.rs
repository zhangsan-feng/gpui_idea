use crate::{data_context::DATA_CONTEXT, gui::main_window::content_area::editor::Editor};
use gpui_kit::{App, Context};

impl Editor {
    pub(in crate::gui::main_window::content_area::editor) fn can_run_current_file(
        &self,
        cx: &App,
    ) -> bool {
        let shell = self.peer_window_bar(cx);
        let tree = self.peer_file_tree(cx);
        !self.state.open_files.is_empty()
            && !self
                .state
                .external_source_files
                .contains(&self.state.selected_file)
            && !shell.read(cx).state.closing
            && !shell.read(cx).state.close_prompt_open
            && !tree.read(cx).state.file_operation_running
            && tree.read(cx).state.moving_path.is_none()
            && !DATA_CONTEXT.project.run_sessions().starting
            && DATA_CONTEXT
                .project
                .file_tree
                .is_file_runnable(self.state.selected_file.relative_path())
    }

    pub(in crate::gui::main_window::content_area::editor) fn run_current_file(
        &self,
        cx: &mut Context<Self>,
    ) {
        if !self.can_run_current_file(cx) {
            return;
        }
        tracing::debug!(file = %self.state.selected_file.relative_path().display(), "editor context menu run requested");
        self.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(
            crate::gui::main_window::content_area::bottom_panel::Event::RunFile(
                self.state.selected_file.relative_path().to_owned(),
            ),
        ));
    }
}
