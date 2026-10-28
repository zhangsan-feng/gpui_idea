use super::*;
use crate::gui::workspace::ToolbarAction;
use gpui_kit::component::menu::PopupMenuItem;
impl BottomPanel {
    pub(in crate::gui::workspace) fn toolbar_item(
        label: impl Into<SharedString>,
        action: ToolbarAction,
        enabled: bool,
        checked: bool,
        view: &WeakEntity<BottomPanel>,
    ) -> PopupMenuItem {
        crate::gui::component::menu::item(
            label,
            enabled,
            checked,
            view,
            move |this, _window, _cx| {
                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                    crate::gui::workspace::window_bar::Event::ToolbarAction(action.clone()),
                ))
            },
        )
    }
}
