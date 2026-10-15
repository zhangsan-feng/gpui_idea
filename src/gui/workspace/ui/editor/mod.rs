use super::*;
use gpui_kit::component::input::Editor;
use std::cell::Cell;

mod tabs;

const TAB_HEIGHT: f32 = 36.;
const TAB_SCROLLBAR_HEIGHT: f32 = 4.;
const TAB_TEXT: u32 = 0xd65360;
const TAB_INDICATOR: u32 = 0xaaaabb;
const TAB_HOVER: u32 = 0xf7f7fa;

impl Workspace {
    pub(in crate::gui::workspace::ui) fn render_editor(&self, cx: &mut Context<Self>) -> Div {
        let mut editor = v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .bg(color(EDITOR_BG));
        if self.open_files.is_empty() {
            return editor
                .child(
                    div()
                        .h(px(TAB_HEIGHT + TAB_SCROLLBAR_HEIGHT + 1.))
                        .flex_shrink_0()
                        .border_b_1()
                        .border_color(color(BORDER))
                        .bg(color(EDITOR_BG)),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(color(MUTED))
                        .child("从项目树选择文件，或将文件拖入这里打开"),
                );
        }
        editor = editor.child(self.editor_tabs(cx));
        if let Some(document) = self.active_document() {
            if let Some(error) = &document.save_error {
                editor = editor.child(
                    div()
                        .px_3()
                        .py_2()
                        .text_size(px(12.))
                        .text_color(color(0xb42318))
                        .bg(color(0xfff0ec))
                        .child(format!("自动保存已暂停：{error}。可按 Ctrl+S 重试。")),
                );
            }
        }
        if let Some(state) = self.editors.get(&self.selected_file) {
            let started = std::time::Instant::now();
            let input = Editor::new(state)
                .aria_label(
                    self.selected_file
                        .relative_path()
                        .to_string_lossy()
                        .into_owned(),
                )
                .appearance(false)
                .bordered(false)
                .readonly(
                    self.closing
                        || self.external_source_files.contains(&self.selected_file)
                        || self.file_operation_running
                        || self.moving_path.as_ref().is_some_and(|source| {
                            self.selected_file.relative_path().starts_with(source)
                        }),
                )
                .h(relative(1.))
                .w_full()
                .font_family("Cascadia Mono")
                .text_size(px(13.));
            editor.child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(super::diagnostics::MeasuredWorkspace::new(
                        "editor",
                        crate::component::code_editor::completion_dropdown(state, input, cx)
                            .into_any_element(),
                        self.window_diagnostics.clone(),
                        started,
                    ))
                    .child(self.editor_wheel_handler(state, cx)),
            )
        } else {
            editor.child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(12.))
                    .text_color(color(MUTED))
                    .child(
                        self.load_errors
                            .get(&self.selected_file)
                            .cloned()
                            .unwrap_or_else(|| "正在打开文件…".into()),
                    ),
            )
        }
    }

    fn editor_wheel_handler(
        &self,
        state: &Entity<gpui_kit::component::input::EditorState>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let state = state.clone();
        let view = cx.weak_entity();
        let file = self.selected_file.clone();
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, cx| {
                if crate::component::code_editor::language_for_path(file.relative_path()) == "rust"
                {
                    crate::component::code_editor::definition_pointer(&state, &hitbox, window, cx);
                }
                let clicked_editor = state.clone();
                let click_hitbox = hitbox.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture
                        || event.button != MouseButton::Left
                        || !event.modifiers.control
                        || event.modifiers.alt
                        || event.modifiers.shift
                        || event.click_count != 1
                        || !click_hitbox.is_hovered(window)
                    {
                        return;
                    }
                    let clicked_editor = clicked_editor.clone();
                    let view = view.clone();
                    let file = file.clone();
                    // Let the editor place its caret with its own scroll/gutter
                    // geometry, then read that position after mouse dispatch.
                    window.defer(cx, move |window, cx| {
                        let state = clicked_editor.read(cx);
                        if !state.focus_handle(cx).is_focused(window)
                            || !state.selected_range().is_empty()
                        {
                            return;
                        }
                        let offset = state.cursor();
                        let _ = view.update(cx, |this, cx| {
                            if this.selected_file == file {
                                this.go_to_definition(file, offset, window, cx);
                            }
                        });
                    });
                });
                let pending_offset = Cell::new(state.read(cx).scroll_offset());
                window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture && hitbox.should_handle_scroll(window) {
                        Self::scroll_editor_lines(&state, event, &pending_offset, cx);
                    }
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }
}
