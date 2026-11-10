use super::CodeSearchWindow;
use gpui_kit::component::{ActiveTheme as _, h_flex, input::Input, v_flex};
use gpui_kit::*;

impl CodeSearchWindow {
    pub(super) fn render_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let hover = theme.list_hover;
        let active = theme.list_active;
        let status = if self.loading {
            "正在搜索…".to_owned()
        } else if let Some(error) = &self.error {
            error.clone()
        } else if self.input.read(cx).value().is_empty() {
            "输入代码内容开始搜索 · 区分大小写".to_owned()
        } else if self.results.is_empty() {
            "没有匹配的代码内容".to_owned()
        } else {
            format!(
                "{} 个匹配行{} · 点击定位，Enter 打开首项",
                self.results.len(),
                if self.truncated {
                    " · 已达到搜索上限"
                } else {
                    ""
                }
            )
        };
        let mut results = v_flex()
            .id("code-search-results")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .gap_1();
        for (index, result) in self.results.iter().enumerate() {
            let result = result.clone();
            results = results.child(
                h_flex()
                    .id(("code-search-result", index))
                    .w_full()
                    .flex_shrink_0()
                    .p_3()
                    .gap_3()
                    .rounded_md()
                    .cursor_pointer()
                    .hover(move |style| style.bg(hover))
                    .active(move |style| style.bg(active))
                    .child(crate::component::project_icon::project_icon(
                        &result.path,
                        false,
                    ))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(div().truncate().child(format!(
                                "{}:{}:{}",
                                result.path.display(),
                                result.line,
                                result.column
                            )))
                            .child(
                                div()
                                    .truncate()
                                    .font_family(crate::component::appearance::editor_font())
                                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                    .text_color(muted)
                                    .child(result.preview.clone()),
                            ),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_match(result.clone(), window, cx)
                    })),
            );
        }
        v_flex()
            .flex_1()
            .min_h_0()
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, window, _| {
                if event.keystroke.key == "escape" {
                    window.remove_window();
                }
            }))
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .p_4()
                    .gap_3()
                    .child(Input::new(&self.input))
                    .child(
                        div()
                            .text_size(crate::gui::component::appearance::ui_text_size(12.))
                            .text_color(if self.error.is_some() {
                                theme.danger
                            } else {
                                muted
                            })
                            .child(status),
                    )
                    .child(results),
            )
    }
}
