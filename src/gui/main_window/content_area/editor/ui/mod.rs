use super::*;
use crate::gui::main_window::content_area::editor::Editor;
use crate::gui::main_window::ui::*;
use gpui_kit::component::{input::Editor as CodeEditor, menu::ContextMenuExt};
use std::cell::Cell;

mod empty;
mod guides;
mod gutter;
mod image_preview;
mod markdown;
mod menu;
mod tabs;

const TAB_HEIGHT: f32 = WORKBENCH_HEADER_HEIGHT - TAB_SCROLLBAR_HEIGHT - 1.;
const TAB_SCROLLBAR_HEIGHT: f32 = 4.;
const TAB_TEXT: u32 = 0xd65360;
const SOURCE_TAB_TEXT: u32 = 0x2563eb;
const SOURCE_TAB_BG: u32 = 0xfff7e6;
const SOURCE_TAB_HOVER: u32 = 0xffefcc;
const TAB_INDICATOR: u32 = 0xaaaabb;
const TAB_HOVER: u32 = 0xf7f7fa;

impl Editor {
    pub(in crate::gui) fn render_editor(
        &self,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) -> Div {
        let mut editor = v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .on_action(cx.listener(|this, _: &super::RunEditorFile, _, cx| {
                this.run_current_file(cx);
            }))
            .on_action(cx.listener(|this, _: &super::RevealEditorFile, _, cx| {
                this.reveal_current_file(cx);
            }))
            .on_action(
                cx.listener(|this, _: &super::CopyEditorProjectPath, _, cx| {
                    this.copy_current_path(false, cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &super::CopyEditorAbsolutePath, _, cx| {
                    this.copy_current_path(true, cx);
                }),
            )
            .bg(crate::component::appearance::background(EDITOR_BG));
        if self.state.open_files.is_empty() {
            if !DATA_CONTEXT.has_project() {
                return editor.child(self.empty_workspace(cx));
            }
            return editor
                .bg(crate::component::appearance::background(PANEL_BG))
                .child(self.empty_editor());
        }
        editor = editor.child(self.editor_tabs(cx));
        if crate::domain::image_preview::PreviewImageFormat::for_path(
            self.state.selected_file.relative_path(),
        )
        .is_some()
        {
            return editor.child(self.render_image_preview(cx));
        }
        if let Some(document) = self.active_document() {
            if let Some(error) = &document.save_error {
                editor = editor.child(
                    div()
                        .px_3()
                        .py_2()
                        .text_size(crate::gui::component::appearance::ui_text_size(12.))
                        .text_color(crate::component::appearance::foreground(0xb42318))
                        .bg(crate::component::appearance::background(0xfff0ec))
                        .child(format!("自动保存已暂停：{error}。可按 Ctrl+S 重试。")),
                );
            }
        }
        if let Some(state) = self.state.editors.get(&self.state.selected_file) {
            let mode = self.markdown_mode();
            if mode == Some(MarkdownMode::Rendered) {
                return editor.child(
                    h_flex()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .overflow_hidden()
                        .child(self.render_markdown(state, window, cx)),
                );
            }
            crate::gui::component::code_editor::update_gutter(state, window, cx);
            let started = std::time::Instant::now();
            let input = CodeEditor::new(state)
                // Keep the input's caret/selection handling, render the shared popup below.
                .context_menu(|_, _, _| gpui_kit::component::native_menu::NativeMenu::new())
                .aria_label(
                    self.state
                        .selected_file
                        .relative_path()
                        .to_string_lossy()
                        .into_owned(),
                )
                .appearance(false)
                .bordered(false)
                .readonly(
                    self.peer_window_bar(cx).read(cx).state.closing
                        || self
                            .state
                            .external_source_files
                            .contains(&self.state.selected_file)
                        || self
                            .peer_file_tree(cx)
                            .read(cx)
                            .state
                            .file_operation_running
                        || self
                            .peer_file_tree(cx)
                            .read(cx)
                            .state
                            .moving_path
                            .as_ref()
                            .is_some_and(|source| {
                                self.state.selected_file.relative_path().starts_with(source)
                            }),
                )
                .h(relative(1.))
                .w_full()
                .font_family(crate::component::appearance::editor_font())
                .text_color(crate::component::appearance::themed(
                    "editor.foreground",
                    crate::component::appearance::foreground(TEXT),
                ))
                .text_size(px(DATA_CONTEXT
                    .settings
                    .snapshot()
                    .general
                    .appearance
                    .editor_size as f32));
            let source = div()
                .h_full()
                .relative()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .child(
                    crate::gui::component::measured_element::MeasuredElement::new(
                        crate::component::code_editor::completion_dropdown(state, input, cx)
                            .into_any_element(),
                        started,
                        {
                            let diagnostics = self
                                .peer_window_bar(cx)
                                .read(cx)
                                .state
                                .window_diagnostics
                                .clone();
                            move |times, window, cx| {
                                diagnostics.borrow_mut().record("editor", times, window, cx)
                            }
                        },
                    ),
                )
                .child(self.gutter_separator(state))
                .child(self.editor_pointer_handler(state, cx))
                .when(mode.is_some(), |source| {
                    self.markdown_drop_target(source, true, cx)
                })
                .context_menu(self.editor_context_menu(state, cx));
            if mode == Some(MarkdownMode::Preview) {
                editor.child(
                    div().flex_1().min_h_0().min_w_0().overflow_hidden().child(
                        gpui_kit::component::resizable::h_resizable("markdown-preview-columns")
                            .with_state(&self.state.markdown_resize)
                            .child(
                                gpui_kit::component::resizable::resizable_panel()
                                    .size_range(px(160.)..Pixels::MAX)
                                    .child(source),
                            )
                            .child(
                                gpui_kit::component::resizable::resizable_panel()
                                    .size_range(px(160.)..Pixels::MAX)
                                    .child(self.render_markdown(state, window, cx)),
                            ),
                    ),
                )
            } else {
                editor.child(source)
            }
        } else {
            editor.child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child(
                        self.state
                            .load_errors
                            .get(&self.state.selected_file)
                            .cloned()
                            .unwrap_or_else(|| "正在打开文件…".into()),
                    ),
            )
        }
    }

    fn editor_pointer_handler(
        &self,
        state: &Entity<gpui_kit::component::input::EditorState>,
        cx: &Context<Editor>,
    ) -> impl IntoElement {
        let state = state.clone();
        let view = cx.weak_entity();
        let file = self.state.selected_file.clone();
        let hitbox_state = state.clone();
        canvas(
            move |bounds, window, cx| {
                let state = hitbox_state.read(cx);
                // Resolve after the native editor's prepaint so opening/closing
                // search uses this frame's text viewport, not last frame's bounds.
                let viewport = state
                    .prepaint_caret_geometry()
                    .map(|geometry| geometry.input_bounds())
                    .unwrap_or_else(|| state.input_bounds());
                window.insert_hitbox(viewport.intersect(&bounds), HitboxBehavior::Normal)
            },
            move |_, hitbox, window, cx| {
                let supports_lsp =
                    crate::domain::language_server::language_for_path(file.relative_path())
                        .is_some();
                if supports_lsp {
                    crate::component::code_editor::definition_pointer(&state, &hitbox, window, cx);
                }
                let clicked_editor = state.clone();
                let click_hitbox = hitbox.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase != DispatchPhase::Capture
                        || !supports_lsp
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
                            if this.state.selected_file == file {
                                this.go_to_definition(file, offset, window, cx);
                            }
                        });
                    });
                });
                let pending_offset = Cell::new(state.read(cx).scroll_offset());
                window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
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
