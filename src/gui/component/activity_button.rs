use super::{icon::icon, tokens::*};
use gpui_kit::{assets::IconName, component::tooltip::Tooltip, prelude::FluentBuilder, *};

pub(crate) fn render(
    name: IconName,
    id: &'static str,
    label: &'static str,
    selected: bool,
    accented: bool,
    running: bool,
) -> Stateful<Div> {
    div()
        .id(id)
        .relative()
        .w(px(32.))
        .h(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .bg(crate::component::appearance::background(if accented {
            ACCENT
        } else if selected {
            0xdfe3e8
        } else {
            ACTIVITY_BG
        }))
        .hover(move |style| {
            style.bg(crate::component::appearance::background(if accented {
                0x2866db
            } else {
                0xe4e7eb
            }))
        })
        .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
        .child(icon(
            name,
            18.,
            if accented {
                0xffffff
            } else if selected {
                TEXT
            } else {
                MUTED
            },
        ))
        .when(running, |button| {
            button.child(
                div()
                    .absolute()
                    .right(px(3.))
                    .top(px(3.))
                    .size(px(6.))
                    .rounded_full()
                    .bg(crate::component::appearance::background(0x59a869)),
            )
        })
}
