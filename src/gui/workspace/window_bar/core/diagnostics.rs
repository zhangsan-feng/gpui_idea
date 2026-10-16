use super::super::*;
use crate::gui::workspace::window_bar::WindowBar;
use std::time::Duration;
impl WindowBar {
    pub(in crate::gui::workspace) fn record_window_bounds(
        &mut self,
        window: &mut Window,
        cx: &mut Context<WindowBar>,
    ) {
        if !self
            .state
            .window_diagnostics
            .borrow_mut()
            .bounds_changed(window)
            || self.state.window_diagnostics_task.is_some()
        {
            return;
        }
        let timer = cx.background_executor().timer(Duration::from_secs(1));
        self.state.window_diagnostics_task = Some(cx.spawn_in(window, async move |this, cx| {
            timer.await;
            let _ = this.update_in(cx, |this, window, cx| {
                if let Some(task) = this.state.window_diagnostics_task.take() {
                    task.detach();
                }
                this.state
                    .window_diagnostics
                    .borrow_mut()
                    .report(window, cx);
            });
        }));
    }
}
