use super::super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use crate::gui::workspace::window_bar::WindowBar;
use std::time::Duration;
impl WindowBar {
    pub(in crate::gui::workspace) const AUTOSAVE_DELAY: Duration = Duration::from_secs(3);
    pub(in crate::gui::workspace) fn schedule_save(&mut self, cx: &mut Context<WindowBar>) {
        if !self.state.session_ready || self.state.closing || self.state.close_prompt_open {
            return;
        }
        self.state.session_revision = self.state.session_revision.saturating_add(1);
        self.state.persistence_timer.take();
        tracing::debug!(
            revision = self.state.session_revision,
            delay_ms = Self::AUTOSAVE_DELAY.as_millis(),
            "workspace and document save scheduled"
        );
        let timer = cx.spawn(async move |this, cx| {
            if let Err(error) = crate::application::lifecycle::wait_timer(
                "workspace-autosave",
                Self::AUTOSAVE_DELAY,
            )
            .await
            {
                tracing::warn!(%error, "workspace autosave timer stopped");
                return;
            }
            let Ok(session) = this.update(cx, |this, cx| {
                if let Some(timer) = this.state.persistence_timer.take() {
                    timer.detach();
                }
                this.workspace_session(cx)
            }) else {
                return;
            };
            tracing::debug!(
                revision = session.revision,
                "workspace and document save timer fired"
            );
            if let Err(error) = run_application(async move {
                APPLICATION.documents.save_dirty().await?;
                APPLICATION.history.save_session(session).await
            })
            .await
            {
                tracing::error!(error = %error, "workspace session save failed");
                let _ = this.update(cx, |this, _cx| {
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                            "工作区状态保存失败：{error:#}"
                        ))),
                    ));
                    crate::event_bus::EVENT_BUS.refresh_gui.notify(&[
                        crate::event_bus::refresh_gui::GuiComponent::Editor,
                        crate::event_bus::refresh_gui::GuiComponent::FileTree,
                        crate::event_bus::refresh_gui::GuiComponent::BottomBar,
                    ]);
                });
            }
        });
        self.state.persistence_timer = Some(timer);
    }
    pub(in crate::gui::workspace) fn resume_autosave(&mut self, cx: &mut Context<WindowBar>) {
        APPLICATION.documents.resume_autosave();
        self.schedule_save(cx);
    }
}
