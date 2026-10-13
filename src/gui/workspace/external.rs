use super::Workspace;
use gpui_kit::{Context, Window};

impl Workspace {
    pub(in crate::gui) fn request_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.request_close_core(window, cx);
    }
}
