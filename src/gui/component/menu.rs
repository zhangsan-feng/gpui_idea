use gpui_kit::{component::menu::PopupMenuItem, *};

/// Bind an enabled menu item to a view without retaining the view.
pub(crate) fn item<T: 'static>(
    label: impl Into<SharedString>,
    enabled: bool,
    checked: bool,
    view: &WeakEntity<T>,
    action: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
) -> PopupMenuItem {
    let view = view.clone();
    PopupMenuItem::new(label)
        .disabled(!enabled)
        .checked(checked)
        .on_click(move |_, window, cx| {
            if enabled {
                let _ = view.update(cx, |this, cx| action(this, window, cx));
            }
        })
}
