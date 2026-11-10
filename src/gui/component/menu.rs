use gpui_kit::{
    assets::IconName,
    component::{
        h_flex,
        menu::{PopupMenu, PopupMenuItem},
    },
    prelude::FluentBuilder,
    *,
};

/// IDEA-style context menu sizing. PopupMenu owns focus, dismissal and keyboard navigation.
pub(crate) fn context_menu(menu: PopupMenu) -> PopupMenu {
    menu.min_w(px(272.)).max_w(px(320.)).scrollable(false)
}

/// Fixed icon and shortcut columns also keep rows without an icon aligned.
pub(crate) fn context_item(
    label: impl Into<SharedString>,
    icon: Option<IconName>,
    shortcut: impl Into<SharedString>,
    enabled: bool,
) -> PopupMenuItem {
    let label = label.into();
    let shortcut = shortcut.into();
    PopupMenuItem::element(move |_, _| {
        let muted = super::appearance::foreground(if enabled { 0x858b98 } else { 0xa8adb8 });
        h_flex()
            .w_full()
            .min_w_0()
            .h(px(26.))
            .items_center()
            .gap(px(8.))
            .font_family(super::appearance::ui_font())
            .text_size(crate::gui::component::appearance::ui_text_size(12.))
            .when(!enabled, |row| row.text_color(muted))
            .child(
                div()
                    .w(px(16.))
                    .flex_shrink_0()
                    .children(icon.map(|name| super::icon::tinted(name, 14., muted))),
            )
            .child(div().flex_1().min_w_0().truncate().child(label.clone()))
            .child(
                div()
                    .flex_shrink_0()
                    .ml(px(16.))
                    .text_color(muted)
                    .child(shortcut.clone()),
            )
    })
    .disabled(!enabled)
}

/// Actions dispatch after PopupMenu restores the target's focus.
pub(crate) fn context_action(
    label: impl Into<SharedString>,
    icon: IconName,
    shortcut: &'static str,
    action: Box<dyn Action>,
    enabled: bool,
) -> PopupMenuItem {
    context_item(label, Some(icon), shortcut, enabled).action(action)
}

pub(crate) fn context_view_item<T: 'static>(
    label: impl Into<SharedString>,
    icon: IconName,
    shortcut: &'static str,
    enabled: bool,
    view: &WeakEntity<T>,
    action: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
) -> PopupMenuItem {
    let view = view.clone();
    context_item(label, Some(icon), shortcut, enabled).on_click(move |_, window, cx| {
        if enabled {
            let _ = view.update(cx, |this, cx| action(this, window, cx));
        }
    })
}

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
