use super::*;

impl Workspace {
    pub(in crate::gui::workspace::ui) fn render_status_bar(&self, cx: &Context<Self>) -> Div {
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
