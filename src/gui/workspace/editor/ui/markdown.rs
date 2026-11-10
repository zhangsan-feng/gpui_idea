use super::*;
use gpui_kit::base::text::{TextView, TextViewState, TextViewStyle};
use gpui_kit::component::tooltip::Tooltip;

impl Editor {
    pub(super) fn markdown_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let current = self.markdown_mode();
        h_flex()
            .mt(px(TAB_SCROLLBAR_HEIGHT))
            .h(px(TAB_HEIGHT))
            .px_1()
            .gap_1()
            .flex_shrink_0()
            .children(
                [
                    (
                        MarkdownMode::Source,
                        IconName::FileText,
                        "原文",
                        "仅显示 Markdown 原文",
                    ),
                    (
                        MarkdownMode::Preview,
                        IconName::Columns2,
                        "预览",
                        "左右分栏，实时预览 Markdown",
                    ),
                    (
                        MarkdownMode::Rendered,
                        IconName::Eye,
                        "渲染",
                        "仅显示渲染后的 Markdown",
                    ),
                ]
                .into_iter()
                .map(|(mode, name, label, tooltip)| {
                    let active = current == Some(mode);
                    h_flex()
                        .id(SharedString::from(format!("markdown-mode-{mode:?}")))
                        .h(px(26.))
                        .px_2()
                        .gap(px(5.))
                        .rounded_sm()
                        .cursor_pointer()
                        .text_size(crate::component::appearance::ui_text_size(12.))
                        .text_color(crate::component::appearance::foreground(if active {
                            ACCENT
                        } else {
                            MUTED
                        }))
                        .when(active, |button| {
                            button.bg(crate::component::appearance::background(0xe8eefb))
                        })
                        .hover(|style| style.bg(crate::component::appearance::background(0xe8eaf0)))
                        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
                        .child(icon(name, 14., if active { ACCENT } else { MUTED }))
                        .child(label)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.set_markdown_mode(mode, window, cx)
                        }))
                }),
            )
    }

    pub(super) fn render_markdown(
        &self,
        state: &Entity<gpui_kit::component::input::EditorState>,
        window: &mut Window,
        cx: &mut App,
    ) -> Div {
        let text = state.read(cx).text().to_string();
        let document_path = self.state.selected_file.path(DATA_CONTEXT.project_root());
        let path = self.state.selected_file.relative_path().to_string_lossy();
        let preview = window.use_keyed_state(
            SharedString::from(format!("markdown-preview-state-{path}")),
            cx,
            |_, cx| TextViewState::markdown("", cx),
        );
        preview.update(cx, |state, cx| state.set_text(&text, cx));
        let line_height = px(DATA_CONTEXT
            .settings
            .snapshot()
            .general
            .appearance
            .editor_size as f32
            * 1.5);
        let pointer_state = preview.clone();
        let style = TextViewStyle::default()
            .with_foreground(crate::component::appearance::foreground(TEXT))
            .with_muted_foreground(crate::component::appearance::foreground(MUTED))
            .with_link(crate::component::appearance::foreground(ACCENT))
            .with_border(crate::component::appearance::border(0xe5e7eb))
            .with_code_background(crate::component::appearance::background(0xf5f6f8));
        div()
            .relative()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(crate::component::appearance::background(EDITOR_BG))
            .pl_5()
            .py_5()
            .text_size(px(DATA_CONTEXT
                .settings
                .snapshot()
                .general
                .appearance
                .editor_size as f32))
            // Reserve a distinct rail at the pane edge. The TextView's own
            // scrollbar stays here while its content uses the padded viewport.
            .child(
                div()
                    .absolute()
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .w(px(14.))
                    .border_l_1()
                    .border_color(crate::component::appearance::border(0xeeeef2))
                    .bg(crate::component::appearance::background(0xf7f7fa)),
            )
            .child(
                TextView::new(&preview)
                    .when_some(
                        self.state
                            .markdown_extensions
                            .get(&self.state.selected_file),
                        |view, extensions| view.markdown_extensions(extensions.clone()),
                    )
                    .image_source(move |uri| Self::markdown_image_source(&document_path, uri))
                    .style(style)
                    .pr(px(34.))
                    .selectable(true)
                    .scrollable(true)
                    .line_height(relative(1.5)),
            )
            .child(
                canvas(
                    |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
                    move |_, hitbox, window, _| {
                        window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
                            if phase == DispatchPhase::Capture
                                && hitbox.should_handle_scroll(window)
                            {
                                Self::scroll_markdown_lines(&pointer_state, event, line_height, cx);
                            }
                        });
                    },
                )
                .absolute()
                .inset_0(),
            )
    }
}

impl gpui_kit::base::text::MarkdownPlugin for MarkdownImages {
    fn is_block(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "document-images"
    }
    fn parse(
        &self,
        node: &gpui_kit::base::text::markdown_ast::Node,
        context: &gpui_kit::base::text::MarkdownParseContext<'_>,
    ) -> Option<gpui_kit::base::text::MarkdownNode> {
        super::super::core::parse_image_paragraph(node, context)
    }
    fn render(
        &self,
        node: &gpui_kit::base::text::MarkdownNode,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        v_flex().w_full().gap_4().children(
            node.data::<Vec<(String, String)>>()
                .into_iter()
                .flatten()
                .map(|(uri, alt)| {
                    let uri = gpui_kit::SharedUri::from(uri.clone());
                    let source = Editor::markdown_image_source(&self.document, &uri);
                    let aspect = Editor::markdown_image_aspect(&source, window, cx);
                    div()
                        .relative()
                        .w_full()
                        .aspect_ratio(aspect)
                        .flex_shrink_0()
                        .child(
                            img(source)
                                .absolute()
                                .size_full()
                                .object_fit(ObjectFit::Contain)
                                .tooltip({
                                    let alt = alt.clone();
                                    move |window, cx| Tooltip::new(alt.clone()).build(window, cx)
                                }),
                        )
                }),
        )
    }
}
