use crate::data_context::DATA_CONTEXT;
mod editor;
mod project;
mod run;
mod toolbar;

use gpui_kit::assets::IconName;
use gpui_kit::component::resizable::{h_resizable, resizable_panel, v_resizable};
use gpui_kit::component::{Icon, Sizable, Size, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::{
    CloseAllDocuments, DraggedEditorTab, DraggedProjectEntry, ProjectFile, SaveDocument, Workspace,
};

const TOOLBAR: u32 = 0x272a31;
const TOOLBAR_HOVER: u32 = 0x3a3e47;
const ACTIVITY_BG: u32 = 0xf1f2f4;
const PANEL_BG: u32 = 0xf6f7f9;
const EDITOR_BG: u32 = 0xffffff;
const BORDER: u32 = 0xdfe2e7;
const TEXT: u32 = 0x20242b;
const MUTED: u32 = 0x737b87;
const ACCENT: u32 = 0x3574f0;
const SELECTED: u32 = 0xdfe1e6;

fn color(value: u32) -> Hsla {
    rgb(value).into()
}

fn icon(name: IconName, size: f32, tint: u32) -> Icon {
    Icon::new(name)
        .with_size(Size::Size(px(size)))
        .text_color(color(tint))
}

impl Workspace {
    pub(super) fn render_view(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .id("workspace")
            .key_context("Workspace")
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                cx.stop_propagation();
                this.open_dropped_files(paths, window, cx);
            }))
            .drag_over::<ExternalPaths>(|style, _, _, _| {
                style.border_1().border_color(color(ACCENT))
            })
            .on_action(
                cx.listener(|this, _: &SaveDocument, window, cx| this.save_current(window, cx)),
            )
            .on_action(cx.listener(|this, _: &CloseAllDocuments, _, cx| {
                this.close_all_editor_tabs(cx);
            }))
            .size_full()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .font_family("Microsoft YaHei UI")
            .text_size(px(12.))
            .bg(color(EDITOR_BG))
            .text_color(color(TEXT))
            .child(self.render_toolbar(window, cx))
            .child(self.render_body(cx))
            .child(self.render_status_bar(cx))
    }

    fn render_body(&self, cx: &mut Context<Self>) -> Div {
        let content =
            if self.run_panel_open {
                v_resizable("workspace-rows")
                .with_state(&self.run_resize)
                .on_resize(cx.listener(
                    |this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                        tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace rows resized");
                        this.schedule_session_save(cx);
                    },
                ))
                .child(
                    resizable_panel()
                        .size_range(px(180.)..Pixels::MAX)
                        .child(self.render_workbench(cx)),
                )
                .child(
                    resizable_panel()
                        .size(px(DATA_CONTEXT.startup().session
                            .as_ref()
                            .and_then(|saved| saved.run_height)
                            .unwrap_or(250.)))
                        .size_range(px(120.)..px(500.))
                        .flex_none()
                        .child(self.render_run_panel(cx)),
                )
                .into_any_element()
            } else {
                self.render_workbench(cx)
            };
        h_flex()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .items_stretch()
            .overflow_hidden()
            .child(self.render_activity_bar(cx))
            .child(div().flex_1().min_w_0().min_h_0().h_full().child(content))
            .child(self.render_right_tool_bar())
    }

    fn render_workbench(&self, cx: &mut Context<Self>) -> AnyElement {
        if !self.project_panel_open {
            return self.render_editor(cx).into_any_element();
        }
        h_resizable("workspace-columns")
            .with_state(&self.project_resize)
            .on_resize(cx.listener(
                |this, state: &Entity<gpui_kit::component::ResizableState>, _, cx| {
                    tracing::debug!(sizes = ?state.read(cx).sizes(), "workspace columns resized");
                    this.schedule_session_save(cx);
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
                    .size_range(px(180.)..px(480.))
                    .flex_none()
                    .child(self.render_project_panel(cx)),
            )
            .child(
                resizable_panel()
                    .size_range(px(360.)..Pixels::MAX)
                    .child(self.render_editor(cx)),
            )
            .into_any_element()
    }

    fn render_activity_bar(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_1()
            .py_2()
            .bg(color(ACTIVITY_BG))
            .border_r_1()
            .border_color(color(BORDER))
            .child(self.activity_button(
                IconName::Folder,
                "project-tool",
                self.project_panel_open,
                cx,
            ))
            .child(self.activity_button(IconName::SlidersHorizontal, "structure-tool", false, cx))
            .child(self.activity_button(IconName::LayoutPanelLeft, "services-tool", false, cx))
            .child(div().flex_1())
            .child(self.activity_button(IconName::Play, "run-tool", self.run_panel_open, cx))
    }

    fn activity_button(
        &self,
        name: IconName,
        id: &'static str,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bg = if selected && id == "run-tool" {
            ACCENT
        } else if selected {
            0xdfe3e8
        } else {
            ACTIVITY_BG
        };
        let button = div()
            .id(id)
            .w(px(34.))
            .h(px(34.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(color(bg))
            .hover(move |style| {
                style.bg(color(if selected && id == "run-tool" {
                    0x2866db
                } else {
                    0xe4e7eb
                }))
            })
            .child(icon(
                name,
                18.,
                if selected && id == "run-tool" {
                    0xffffff
                } else if selected {
                    TEXT
                } else {
                    MUTED
                },
            ));
        if id == "project-tool" {
            button.on_click(cx.listener(|this, _, _, cx| this.toggle_project_panel(cx)))
        } else if id == "run-tool" {
            button.on_click(cx.listener(|this, _, _, cx| this.toggle_run_panel(cx)))
        } else {
            button
        }
    }
    fn render_right_tool_bar(&self) -> Div {
        v_flex()
            .w(px(40.))
            .h_full()
            .flex_shrink_0()
            .items_center()
            .gap_2()
            .py_2()
            .bg(color(PANEL_BG))
            .border_l_1()
            .border_color(color(BORDER))
            .child(icon(IconName::BellDot, 17., MUTED))
            .child(icon(IconName::Workflow, 17., MUTED))
            .child(icon(IconName::Database, 17., MUTED))
            .child(icon(IconName::Code, 17., MUTED))
    }
    fn render_status_bar(&self, cx: &Context<Self>) -> Div {
        let cursor = self
            .editors
            .get(&self.selected_file)
            .filter(|_| !self.open_files.is_empty())
            .map(|editor| editor.read(cx).cursor_position());
        let position = cursor
            .map(|point| format!("{}:{}", point.line + 1, point.character + 1))
            .unwrap_or_else(|| "—".into());
        let line_ending = self
            .active_document()
            .map(|doc| match doc.format.line_ending {
                crate::domain::document::LineEnding::Lf => "LF",
                crate::domain::document::LineEnding::CrLf => "CRLF",
            })
            .unwrap_or("LF");
        let autosave_status = self
            .active_document()
            .map(|document| {
                if document.save_error.is_some() {
                    "自动保存已暂停"
                } else if document.saving {
                    "保存中"
                } else if document.dirty() {
                    "等待自动保存"
                } else {
                    "自动保存"
                }
            })
            .unwrap_or("自动保存");
        h_flex()
            .w_full()
            .h(px(25.))
            .flex_shrink_0()
            .items_center()
            .gap_3()
            .px_2()
            .border_t_1()
            .border_color(color(BORDER))
            .bg(color(PANEL_BG))
            .text_size(px(11.))
            .text_color(color(0x626a76))
            .child(icon(IconName::Folder, 13., ACCENT))
            .child(
                div().child(
                    DATA_CONTEXT
                        .project_root()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                ),
            )
            .child(icon(IconName::ChevronRight, 11., 0x9ca3ad))
            .child(div().child(if !self.open_files.is_empty() {
                self.selected_file.name()
            } else {
                String::new()
            }))
            .child(div().flex_1())
            .child(
                div()
                    .max_w(px(360.))
                    .truncate()
                    .child(self.notice.clone().unwrap_or_default()),
            )
            .child(div().child(position))
            .child(div().child(autosave_status))
            .child(div().child(line_ending))
            .child(div().child("UTF-8"))
            .child(div().child("4 spaces"))
            .child(div().w(px(1.)).h(px(14.)).bg(color(BORDER)))
            .child(div().child("GPUI Kit"))
    }
}
