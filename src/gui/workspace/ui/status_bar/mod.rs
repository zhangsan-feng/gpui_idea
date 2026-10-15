use super::*;
use crate::domain::language_server::LanguageServerStatus;
use gpui_kit::component::tooltip::Tooltip;

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
                if self.external_source_files.contains(&self.selected_file) {
                    "依赖源码 · 只读"
                } else if document.save_error.is_some() {
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
        let status = DATA_CONTEXT.language_server_status();
        let (lsp_label, lsp_color, lsp_detail) = match status {
            LanguageServerStatus::Stopped => {
                ("LSP 已停止", MUTED, "rust-analyzer 已停止".to_owned())
            }
            LanguageServerStatus::Starting => (
                "LSP 启动中",
                ACCENT,
                "正在启动并初始化 rust-analyzer".to_owned(),
            ),
            LanguageServerStatus::Indexing => (
                "LSP 索引中",
                ACCENT,
                "rust-analyzer 正在加载工作区和索引源码".to_owned(),
            ),
            LanguageServerStatus::Ready => {
                ("LSP 就绪", 0x16a34a, "rust-analyzer 已就绪".to_owned())
            }
            LanguageServerStatus::Warming => (
                "LSP 预热中",
                ACCENT,
                "正在后台分析已打开的 Rust 文档，预热源码跳转缓存".to_owned(),
            ),
            LanguageServerStatus::Warning(detail) => ("LSP 警告", 0xd97706, detail),
            LanguageServerStatus::Error(detail) => ("LSP 异常", 0xdc2626, detail),
        };
        h_flex()
            .w_full()
            .h(px(32.))
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
                h_flex()
                    .id("language-server-status")
                    .flex_shrink_0()
                    .gap_1p5()
                    .text_color(color(lsp_color))
                    .child(div().size(px(6.)).rounded_full().bg(color(lsp_color)))
                    .child(lsp_label)
                    .tooltip(move |window, cx| Tooltip::new(lsp_detail.clone()).build(window, cx)),
            )
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
            .when(!self.open_files.is_empty(), |bar| {
                bar.child(div().child(crate::component::code_editor::language_label(
                    self.selected_file.relative_path(),
                )))
            })
            .child(div().child("4 spaces"))
            .child(div().w(px(1.)).h(px(14.)).bg(color(BORDER)))
            .child(div().child("GPUI Kit"))
    }
}
