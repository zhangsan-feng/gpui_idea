use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::{AppContext as _, Context, Window, px, size};

impl WindowBar {
    pub(in crate::gui::workspace) fn open_settings(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WindowBar>,
    ) {
        if self.state.closing || self.state.close_prompt_open {
            return;
        }
        if let Some(handle) = self.state.settings_window {
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        match crate::gui::external::open_tool_window(
            "设置",
            size(px(1120.), px(680.)),
            window,
            cx,
            |window, cx| {
                cx.new(|cx| crate::gui::workspace::settings::WorkspaceSettings::new(window, cx))
            },
        ) {
            Ok((handle, _)) => self.state.settings_window = Some(handle),
            Err(error) => {
                self.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                    crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                        "无法打开配置窗口：{error:#}"
                    ))),
                ));
                crate::event_bus::EVENT_BUS
                    .refresh_gui
                    .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
            }
        }
    }
}
