use super::*;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::component::menu::ContextMenuExt;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::tooltip::Tooltip;

impl Workspace {
    pub(super) fn editor_tabs(&self, cx: &mut Context<Self>) -> Div {
        let document = self.active_document();
        let dirty = document.as_ref().is_some_and(|doc| doc.dirty());
        h_flex()
            .h(px(TAB_HEIGHT + TAB_SCROLLBAR_HEIGHT + 1.))
            .min_w_0()
            .flex_shrink_0()
            .items_start()
            .border_b_1()
            .border_color(color(0xeeeef2))
            .bg(color(EDITOR_BG))
            .child(
                v_flex()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .id("editor-tabs-scrollbar")
                            .relative()
                            .w_full()
                            .h(px(TAB_SCROLLBAR_HEIGHT))
                            .flex_shrink_0()
                            .child(
                                Scrollbar::horizontal(&self.editor_tabs_scroll)
                                    .viewport_from_layout()
                                    .styles(|styles| {
                                        styles
                                            .track(|track| track.width(px(TAB_SCROLLBAR_HEIGHT)))
                                            .thumb(|thumb| {
                                                thumb
                                                    .width(px(TAB_SCROLLBAR_HEIGHT))
                                                    .inset(px(0.))
                                                    .radius(px(0.))
                                                    .bg(color(0x898989))
                                            })
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .id("editor-tabs")
                            .h(px(TAB_HEIGHT))
                            .w_full()
                            .flex_shrink_0()
                            .min_w_0()
                            .items_center()
                            .overflow_x_scroll()
                            .track_scroll(&self.editor_tabs_scroll)
                            .children(self.open_files.iter().map(|file| self.editor_tab(file, cx))),
                    ),
            )
            .child(
                div()
                    .id("save-document")
                    .w(px(36.))
                    .h(px(TAB_HEIGHT))
                    .mt(px(TAB_SCROLLBAR_HEIGHT))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .hover(|style| style.bg(color(0xe8eaf0)))
                    .tooltip(|window, cx| Tooltip::new("保存当前文件 (Ctrl+S)").build(window, cx))
                    .child(icon(
                        IconName::Save,
                        16.,
                        if dirty { ACCENT } else { MUTED },
                    ))
                    .on_click(cx.listener(|this, _, window, cx| this.save_current(window, cx))),
            )
    }

    fn editor_tab(&self, file: &ProjectFile, cx: &mut Context<Self>) -> AnyElement {
        let active = self.selected_file == *file;
        let document = DATA_CONTEXT
            .documents
            .snapshot(&file.path(DATA_CONTEXT.project_root()));
        let dirty = document.as_ref().is_some_and(|doc| doc.dirty());
        let saving = document.as_ref().is_some_and(|doc| doc.saving);
        let failed = self.load_errors.contains_key(file)
            || document
                .as_ref()
                .is_some_and(|doc| doc.save_error.is_some());
        let selected_file = file.clone();
        let closed_file = file.clone();
        let target_file = file.clone();
        let label = self.editor_tab_label(file);
        let path = file.relative_path().to_string_lossy().into_owned();
        let tooltip_path = path.clone();
        let group_id = SharedString::from(format!("editor-tab-{path}"));
        let close_all_disabled = self.closing || self.close_prompt_open;
        h_flex()
            .id(group_id.clone())
            .group(group_id.clone())
            .relative()
            .h_full()
            .flex_shrink_0()
            .max_w(px(280.))
            .pl(px(10.))
            .pr(px(6.))
            .gap(px(6.))
            .items_center()
            .bg(color(EDITOR_BG))
            .text_size(px(12.))
            .text_color(color(TAB_TEXT))
            .cursor_pointer()
            .hover(|style| style.bg(color(TAB_HOVER)))
            .tooltip(move |window, cx| Tooltip::new(tooltip_path.clone()).build(window, cx))
            .child(crate::component::project_icon::project_icon(file.relative_path(), false))
            .child(div().min_w_0().truncate().child(label.clone()))
            .when(dirty, |tab| tab.child(div().text_color(color(ACCENT)).child("●")))
            .when(saving, |tab| tab.child(div().text_color(color(MUTED)).child("保存中")))
            .when(self.loading_files.contains(file), |tab| tab.child(div().text_color(color(MUTED)).child("打开中")))
            .when(failed, |tab| tab.child(icon(IconName::CircleAlert, 13., 0xb42318)))
            .child(
                div()
                    .id(SharedString::from(format!("close-editor-tab-{path}")))
                    .w(px(20.))
                    .h(px(20.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .when(!active, |button| {
                        button.invisible().group_hover(group_id, |style| style.visible())
                    })
                    .hover(|style| style.bg(color(0xeaeaf0)))
                    .tooltip(|window, cx| Tooltip::new("关闭文件").build(window, cx))
                    .child(icon(IconName::X, 12., TAB_INDICATOR))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.close_editor_tab(&closed_file, window, cx);
                    })),
            )
            .when(active, |tab| {
                tab.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left(px(10.))
                        .right(px(10.))
                        .h(px(3.))
                        .rounded_full()
                        .bg(color(TAB_INDICATOR)),
                )
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_file(selected_file.clone(), window, cx);
            }))
            .on_drag(DraggedEditorTab { file: file.clone(), label }, |drag, _, _, cx| {
                tracing::debug!(file = %drag.file.relative_path().display(), "editor tab drag started");
                cx.new(|_| drag.clone())
            })
            .drag_over::<DraggedEditorTab>(|style, _, _, _| style.bg(color(0xeeeef5)))
            .on_drop(cx.listener(move |this, drag: &DraggedEditorTab, window, cx| {
                cx.stop_propagation();
                this.move_editor_tab(&drag.file, &target_file, window, cx);
            }))
            .context_menu(move |menu, _, _| {
                menu.menu_with_disabled("关闭所有", Box::new(CloseAllDocuments), close_all_disabled)
            })
            .into_any_element()
    }
}

impl Render for DraggedEditorTab {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h(px(36.))
            .max_w(px(280.))
            .px_3()
            .gap_2()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(color(ACCENT))
            .bg(color(EDITOR_BG))
            .shadow_md()
            .font_family("Microsoft YaHei UI")
            .text_size(px(12.))
            .text_color(color(TAB_TEXT))
            .child(crate::component::project_icon::project_icon(
                self.file.relative_path(),
                false,
            ))
            .child(div().min_w_0().truncate().child(self.label.clone()))
    }
}
