use super::*;
use gpui_kit::component::{WindowExt as _, input::Input};

impl Workspace {
    pub(in crate::gui::workspace) fn render_file_search(
        &self,
        view: WeakEntity<Self>,
        cx: &App,
    ) -> Div {
        let mut content = v_flex().gap_2().child(Input::new(&self.search_input));
        let status = if self.search_loading {
            "正在搜索…".into()
        } else if let Some(error) = &self.search_error {
            error.clone()
        } else if self.search_input.read(cx).value().is_empty() {
            "输入代码内容开始搜索 · 区分大小写".into()
        } else if self.search_results.is_empty() {
            "没有匹配的代码内容".into()
        } else {
            format!(
                "{} 个匹配行 · 最多显示 100 个 · 点击定位，Enter 打开首项",
                self.search_results.len()
            )
        };
        content = content.child(
            div()
                .text_color(color(MUTED))
                .text_size(px(12.))
                .child(status),
        );
        let mut results = v_flex()
            .id("file-search-results")
            .h(px(340.))
            .overflow_y_scroll()
            .gap_1();
        for (index, result) in self.search_results.iter().enumerate() {
            let result = result.clone();
            let view = view.clone();
            results = results.child(
                h_flex()
                    .id(("file-search-result", index))
                    .p_2()
                    .gap_2()
                    .cursor_pointer()
                    .rounded_sm()
                    .hover(|s| s.bg(color(0xe8eefb)))
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
                                    .font_family("Cascadia Mono")
                                    .text_size(px(12.))
                                    .text_color(color(MUTED))
                                    .child(result.preview.clone()),
                            ),
                    )
                    .on_click(move |_, window, cx| {
                        window.close_dialog(cx);
                        let _ = view.update(cx, |this, cx| {
                            this.search_request += 1;
                            this.open_content_match(result.clone(), window, cx);
                        });
                    }),
            );
        }
        content.child(results)
    }
}
