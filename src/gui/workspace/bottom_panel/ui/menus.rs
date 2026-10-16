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
        let view = view.clone();
        PopupMenuItem::new(label)
            .disabled(!enabled)
            .checked(checked)
            .on_click(move |_, _window, cx| {
                if enabled {
                    let _ = view.update(cx, |this, _cx| {
                        {
                            this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                                crate::gui::workspace::window_bar::Event::ToolbarAction(
                                    action.clone(),
                                ),
                            ))
                        }
                    });
                }
            })
    }
}
