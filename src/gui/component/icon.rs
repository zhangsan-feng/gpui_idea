use gpui_kit::{
    assets::IconName,
    component::{Icon, Sizable, Size},
    *,
};

pub(crate) fn sized(name: IconName, size: f32) -> Icon {
    Icon::new(name).with_size(Size::Size(px(size)))
}

pub(crate) fn tinted(name: IconName, size: f32, tint: Hsla) -> Icon {
    sized(name, size).text_color(tint)
}

pub(crate) fn icon(name: IconName, size: f32, tint: u32) -> Icon {
    tinted(name, size, super::appearance::foreground(tint))
}
