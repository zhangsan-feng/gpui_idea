use super::*;
use crate::gui::workspace::editor::Editor;

impl Editor {
    pub(super) fn render_image_preview(&self) -> Div {
        let preview = self.state.image_previews.get(&self.state.selected_file);
        let error = self
            .state
            .load_errors
            .get(&self.state.selected_file)
            .or_else(|| preview.and_then(|view| view.error.as_ref()))
            .cloned();
        crate::gui::component::image_preview::render(
            preview.and_then(|view| view.image.clone()),
            preview.map(|view| view.bytes),
            error,
        )
    }
}
