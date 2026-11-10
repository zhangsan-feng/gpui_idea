use gpui_kit::component::{ActiveTheme as _, v_flex};
use gpui_kit::*;

/// Shared outer frame; each window supplies its own content and close policy.
pub(crate) fn frame(title: &'static str, close_enabled: bool, cx: &App) -> Div {
    let theme = cx.theme();
    v_flex()
        .size_full()
        .min_h_0()
        .min_w_0()
        .font_family(super::super::appearance::ui_font())
        .bg(theme.background)
        .text_color(theme.foreground)
        .text_size(crate::gui::component::appearance::ui_text_size(13.))
        .child(super::super::operation_title_bar::render(
            title,
            close_enabled,
            cx,
        ))
}

pub(crate) fn content() -> Div {
    div().flex_1().min_h_0().min_w_0()
}
