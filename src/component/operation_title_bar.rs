use gpui_kit::assets::IconName;
use gpui_kit::component::{ActiveTheme as _, Icon, Sizable as _, Size, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

pub(crate) const HEIGHT: f32 = 38.;

pub(crate) fn render(title: &'static str, close_enabled: bool, cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let foreground = theme.foreground;
    let muted = theme.muted_foreground;
    let danger = theme.danger;
    h_flex()
        .id("operation-title-bar")
        .w_full()
        .h(px(HEIGHT))
        .flex_shrink_0()
        .items_center()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.sidebar)
        .child(
            h_flex()
                .id("operation-title-bar-drag")
                .flex_1()
                .min_w_0()
                .h_full()
                .px_4()
                .items_center()
                .window_control_area(WindowControlArea::Drag)
                .child(
                    div()
                        .truncate()
                        .text_size(px(13.))
                        .font_weight(FontWeight(600.))
                        .text_color(foreground)
                        .child(title),
                ),
        )
        .child(
            div()
                .id("operation-title-bar-close")
                .w(px(42.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(if close_enabled { foreground } else { muted })
                .when(close_enabled, |button| {
                    button
                        .cursor_pointer()
                        .hover(move |style| style.bg(danger).text_color(gpui_kit::white()))
                        .active(move |style| style.bg(danger).text_color(gpui_kit::white()))
                })
                .child(Icon::new(IconName::X).with_size(Size::Size(px(15.))))
                .on_click(move |_, window, _| {
                    if close_enabled {
                        tracing::debug!(title, "operation window custom close requested");
                        window.remove_window();
                    }
                }),
        )
}
