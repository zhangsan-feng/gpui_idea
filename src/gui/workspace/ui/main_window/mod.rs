use super::*;
use gpui_kit::component::resizable::{h_resizable, resizable_panel};

impl Workspace {
    pub(in crate::gui::workspace) fn render_view(
        &self,
        _window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> impl IntoElement {
        v_flex()
            .id("workspace")
            .key_context("Workspace")
            .on_drop(cx.listener(|_this, paths: &ExternalPaths, _window, cx| {
                cx.stop_propagation();
                crate::event_bus::EVENT_BUS.gui_event.send(
                    cx.entity().entity_id(),
                    crate::event_bus::gui_event::GuiEvent::Editor(
                        crate::gui::workspace::editor::Event::OpenDroppedFiles(paths.clone()),
                    ),
                );
            }))
            .drag_over::<ExternalPaths>(|style, _, _, _| {
                style
                    .border_1()
                    .border_color(crate::component::appearance::border(ACCENT))
            })
            .on_action(cx.listener(|_this, _: &SaveDocument, _window, cx| {
                crate::event_bus::EVENT_BUS.gui_event.send(
                    cx.entity().entity_id(),
                    crate::event_bus::gui_event::GuiEvent::Editor(
                        crate::gui::workspace::editor::Event::SaveCurrent,
                    ),
                )
            }))
            .on_action(cx.listener(|_this, _: &CloseAllDocuments, _, cx| {
                crate::event_bus::EVENT_BUS.gui_event.send(
                    cx.entity().entity_id(),
                    crate::event_bus::gui_event::GuiEvent::Editor(
                        crate::gui::workspace::editor::Event::CloseAllEditorTabs,
                    ),
                );
            }))
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .font_family(crate::component::appearance::ui_font())
            .text_size(px(12.))
            .bg(crate::component::appearance::background(EDITOR_BG))
            .text_color(crate::component::appearance::foreground(TEXT))
            // GPUI's cached paint replay omits window_control_hitboxes. Keep the
            // title bar uncached so native drag/minimize/maximize regions are
            // registered on every frame, including editor-only redraws.
            .child(self.window_bar.clone())
            .child(self.render_body(cx))
            .child(
                AnyView::from(self.bottom_bar.clone()).cached(
                    StyleRefinement::default()
                        .w_full()
                        .h(px(32.))
                        .flex_shrink_0(),
                ),
            )
    }

    fn render_body(&self, cx: &mut Context<Workspace>) -> Div {
        h_flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .items_stretch()
            .overflow_hidden()
            .child(
                AnyView::from(self.left_sidebar.clone()).cached(
                    StyleRefinement::default()
                        .w(px(40.))
                        .h_full()
                        .flex_shrink_0(),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .h_full()
                    .child(self.render_bottom_area(cx)),
            )
            .child(
                AnyView::from(self.right_sidebar.clone()).cached(
                    StyleRefinement::default()
                        .w(px(40.))
                        .h_full()
                        .flex_shrink_0(),
                ),
            )
    }

    pub(in crate::gui::workspace) fn render_workbench(
        &self,
        cx: &mut Context<Workspace>,
    ) -> AnyElement {
        if !self.left_sidebar.read(cx).state.project_panel_open {
            return AnyView::from(self.editor.clone())
                .cached(StyleRefinement::default().size_full())
                .into_any_element();
        }
        h_resizable("workspace-columns")
            .with_state(&self.left_sidebar.read(cx).state.project_resize)
            .on_resize(cx.listener(
                |_this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                    tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace columns resized");
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        cx.entity().entity_id(),
                        crate::event_bus::gui_event::GuiEvent::WindowBar(
                            crate::gui::workspace::window_bar::Event::ScheduleSave,
                        ),
                    );
                },
            ))
            .child(
                resizable_panel()
                    .size(px(DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.project_width)
                        .unwrap_or(300.)))
                    .size_range(px(120.)..px(1024.))
                    // Keep the restored pixel width when startup/maximize changes
                    // the container size; the editor receives the extra space.
                    .fixed()
                    .flex_none()
                    .child(
                        AnyView::from(self.file_tree.clone())
                            .cached(StyleRefinement::default().size_full()),
                    ),
            )
            .child(resizable_panel().size_range(px(360.)..Pixels::MAX).child(
                AnyView::from(self.editor.clone()).cached(StyleRefinement::default().size_full()),
            ))
            .into_any_element()
    }
}

impl Workspace {
    pub(in crate::gui::workspace) fn render_bottom_area(
        &self,
        cx: &mut Context<Workspace>,
    ) -> AnyElement {
        if !self.left_sidebar.read(cx).state.run_panel_open {
            return self.render_workbench(cx);
        }

        gpui_kit::component::resizable::v_resizable("workspace-rows")
            .with_state(&self.left_sidebar.read(cx).state.run_resize)
            .on_resize(cx.listener(
                |_this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                    tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace rows resized");
                    crate::event_bus::EVENT_BUS.gui_event.send(
                        cx.entity().entity_id(),
                        crate::event_bus::gui_event::GuiEvent::WindowBar(
                            crate::gui::workspace::window_bar::Event::ScheduleSave,
                        ),
                    );
                },
            ))
            .child(
                resizable_panel()
                    .size_range(px(180.)..Pixels::MAX)
                    .child(self.render_workbench(cx)),
            )
            .child(
                resizable_panel()
                    .size(px(DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.run_height)
                        .unwrap_or(250.)))
                    .size_range(px(120.)..px(500.))
                    .fixed()
                    .flex_none()
                    .child(
                        AnyView::from(self.bottom_panel.clone())
                            .cached(StyleRefinement::default().size_full()),
                    ),
            )
            .into_any_element()
    }
}
