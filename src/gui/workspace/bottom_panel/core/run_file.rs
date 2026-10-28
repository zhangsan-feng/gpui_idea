use crate::{
    application::{APPLICATION, lifecycle::run_application},
    gui::workspace::bottom_panel::{BottomPanel, ToolWindow},
};
use gpui_kit::Context;
use std::path::PathBuf;

impl BottomPanel {
    pub(in crate::gui::workspace) fn run_file(&mut self, file: PathBuf, cx: &mut Context<Self>) {
        if self.peer_window_bar(cx).read(cx).state.closing
            || self.peer_window_bar(cx).read(cx).state.close_prompt_open
        {
            return;
        }
        self.state.active_tool = ToolWindow::Run;
        self.send(crate::event_bus::gui_event::GuiEvent::LeftSidebar(
            crate::gui::workspace::left_sidebar::Event::SetRunPanel(true),
        ));
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.run_file(file).await }).await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    tracing::warn!(%error, "file tree run failed");
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                            "无法运行：{error:#}"
                        ))),
                    ));
                }
                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                    crate::gui::workspace::window_bar::Event::ScheduleSave,
                ));
                crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                    crate::event_bus::refresh_gui::GuiComponent::WindowBar,
                    crate::event_bus::refresh_gui::GuiComponent::BottomPanel,
                    crate::event_bus::refresh_gui::GuiComponent::LeftSidebar,
                    crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                ]);
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}
