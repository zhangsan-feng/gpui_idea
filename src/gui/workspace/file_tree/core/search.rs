use crate::gui::workspace::file_tree::FileTree;
use gpui_kit::{Context, Window};

impl FileTree {
    pub(in crate::gui::workspace) fn open_file_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(handle) = self.state.search_window {
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        match crate::gui::code_search_window::open(self.workspace.clone(), window, cx) {
            Ok(handle) => self.state.search_window = Some(handle),
            Err(error) => {
                tracing::warn!(%error, "code search window opening failed");
                self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                    crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                        "无法打开代码搜索窗口：{error:#}"
                    ))),
                ));
            }
        }
    }
}
