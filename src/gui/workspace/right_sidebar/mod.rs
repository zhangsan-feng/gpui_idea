mod ui;
use crate::gui::workspace::Workspace;
use crate::gui::workspace::ui::*;
pub(crate) struct RightSidebar {
    subscription: Option<Task<()>>,
}
impl RightSidebar {
    pub(super) fn new(
        _workspace: WeakEntity<Workspace>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut refresh = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::RightSidebar);
        let subscription = cx.spawn(async move |view, cx| {
            while refresh.changed().await.is_ok() {
                if view.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });
        Self {
            subscription: Some(subscription),
        }
    }
}
impl Render for RightSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let _ = &self.subscription;
        self.render_right_tool_bar()
    }
}
