use crate::domain::language_server::LanguageServerStatus;
use crate::gui::workspace::bottom_bar::BottomBar;
use crate::gui::workspace::ui::*;
use gpui_kit::base::Popover;

mod lsp;

impl BottomBar {
    pub(in crate::gui::workspace) fn render_status_bar(&self, cx: &Context<BottomBar>) -> Div {
        let image = !self.peer_editor(cx).read(cx).state.open_files.is_empty()
            && crate::domain::image_preview::PreviewImageFormat::for_path(
                self.peer_editor(cx)
                    .read(cx)
                    .state
                    .selected_file
                    .relative_path(),
            )
            .is_some();
        let cursor = self
            .peer_editor(cx)
            .read(cx)
            .state
            .editors
            .get(&self.peer_editor(cx).read(cx).state.selected_file)
            .filter(|_| !self.peer_editor(cx).read(cx).state.open_files.is_empty())
            .map(|editor| editor.read(cx).cursor_position());
        let position = cursor
            .map(|point| format!("{}:{}", point.line + 1, point.character + 1))
            .unwrap_or_else(|| "—".into());
        let line_ending = self
            .active_document(cx)
            .map(|doc| match doc.format.line_ending {
                crate::domain::document::LineEnding::Lf => "LF",
                crate::domain::document::LineEnding::CrLf => "CRLF",
            })
            .unwrap_or("LF");
        let autosave_status = if image {
            "图片预览 · 只读"
        } else {
            self.active_document(cx)
                .map(|document| {
                    if self
                        .peer_editor(cx)
                        .read(cx)
                        .state
                        .external_source_files
                        .contains(&self.peer_editor(cx).read(cx).state.selected_file)
                    {
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
                .unwrap_or("自动保存")
        };
        let status = DATA_CONTEXT.language_server_status();
        let installation = lsp::installation_detail();
        let installing = installation.is_some();
        let (lsp_label, lsp_color, lsp_detail) = if installing {
            ("LSP 安装中", ACCENT, installation.unwrap_or_default())
        } else {
            match status {
                LanguageServerStatus::Stopped => ("LSP 已停止", MUTED, "语言服务未启动".to_owned()),
                LanguageServerStatus::Starting => {
                    ("LSP 启动中", ACCENT, "正在启动并初始化语言服务".to_owned())
                }
                LanguageServerStatus::Preparing(detail) => ("LSP 准备中", ACCENT, detail),
                LanguageServerStatus::Indexing => (
                    "LSP 索引中",
                    ACCENT,
                    "语言服务正在加载工作区和索引源码".to_owned(),
                ),
                LanguageServerStatus::Ready => ("LSP 就绪", 0x16a34a, "语言服务已就绪".to_owned()),
                LanguageServerStatus::Warming => (
                    "LSP 预热中",
                    ACCENT,
                    "正在后台分析已打开的 Rust 文档，预热源码跳转缓存".to_owned(),
                ),
                LanguageServerStatus::Warning(detail) => ("LSP 警告", 0xd97706, detail),
                LanguageServerStatus::Error(detail) => ("LSP 异常", 0xdc2626, detail),
            }
        };
        let install_detail = installing.then(|| lsp_detail.clone());
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
            .child(div().child(if DATA_CONTEXT.has_project() {
                DATA_CONTEXT
                    .project_root()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            } else {
                "未打开项目".to_owned()
            }))
            .child(icon(IconName::ChevronRight, 11., 0x9ca3ad))
            .child(div().child(
                if !self.peer_editor(cx).read(cx).state.open_files.is_empty() {
                    self.peer_editor(cx).read(cx).state.selected_file.name()
                } else {
                    String::new()
                },
            ))
            .child(div().flex_1())
            .child(
                Popover::new("language-server-popover")
                    .anchor(Anchor::BottomRight)
                    .offset(px(8.))
                    .trigger_with(move |open, _, _| {
                        h_flex()
                            .id("language-server-status")
                            .flex_shrink_0()
                            .items_center()
                            .gap_1p5()
                            .px_1p5()
                            .py_1()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|style| style.bg(color(0xe8eaf0)))
                            .when(open, |indicator| indicator.bg(color(0xe8eaf0)))
                            .text_color(color(lsp_color))
                            .child(div().size(px(6.)).rounded_full().bg(color(lsp_color)))
                            .child(lsp_label)
                            .when_some(install_detail, |indicator, detail| {
                                indicator.child(div().max_w(px(240.)).truncate().child(detail))
                            })
                            .child(icon(
                                if open {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronUp
                                },
                                11.,
                                lsp_color,
                            ))
                            .into_any_element()
                    })
                    .content(move |_, _, _| lsp::details(&lsp_detail)),
            )
            .child(
                div()
                    .max_w(px(360.))
                    .truncate()
                    .child(self.state.notice.clone().unwrap_or_default()),
            )
            .when(!image, |bar| bar.child(div().child(position)))
            .child(div().child(autosave_status))
            .when(!image, |bar| {
                bar.child(div().child(line_ending))
                    .child(div().child("UTF-8"))
            })
            .when(
                !self.peer_editor(cx).read(cx).state.open_files.is_empty() && !image,
                |bar| {
                    bar.child(
                        div().child(crate::component::code_editor::language_label(
                            self.peer_editor(cx)
                                .read(cx)
                                .state
                                .selected_file
                                .relative_path(),
                        )),
                    )
                },
            )
            .when(!image, |bar| bar.child(div().child("4 spaces")))
            .child(div().w(px(1.)).h(px(14.)).bg(color(BORDER)))
            .child(div().child("GPUI Kit"))
    }
}
