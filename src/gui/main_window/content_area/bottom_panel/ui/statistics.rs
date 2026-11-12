use super::*;
use crate::domain::statistics::{DirectoryStatistics, ExtensionStatistics};
use crate::gui::main_window::content_area::bottom_panel::BottomPanel;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::tooltip::Tooltip;

impl BottomPanel {
    pub(super) fn render_statistics_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let snapshot = DATA_CONTEXT.project.statistics();
        let mut rows = Vec::new();
        for (directory_index, directory) in snapshot.directories.iter().enumerate() {
            rows.push((directory_index, None));
            if !self.state.statistics_collapsed.contains(&directory.path) {
                rows.extend(
                    (0..directory.extensions.len()).map(|index| (directory_index, Some(index))),
                );
            }
        }
        let data = snapshot.clone();
        let body = uniform_list(
            "statistics-directory-list",
            rows.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                range
                    .filter_map(|index| {
                        let (directory_index, extension_index) = rows.get(index)?;
                        let directory = data.directories.get(*directory_index)?;
                        Some(match extension_index {
                            Some(index) => this.statistics_type(&directory.extensions[*index]),
                            None => this.statistics_directory(directory, *directory_index, cx),
                        })
                    })
                    .collect::<Vec<_>>()
            }),
        )
        .with_sizing_behavior(ListSizingBehavior::Auto)
        .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::FitList)
        .flex_1()
        .min_h_0()
        .w_full()
        .track_scroll(&self.state.statistics_scroll);

        v_flex()
            .size_full()
            .min_w_0()
            .min_h_0()
            .border_t_1()
            .border_color(crate::component::appearance::border(BORDER))
            .bg(crate::component::appearance::background(PANEL_BG))
            .child(self.tool_header(ToolWindow::Statistics, "目录统计", IconName::ChartNoAxesCombined, cx))
            .child(
                h_flex()
                    .h(px(38.))
                    .flex_shrink_0()
                    .px_3()
                    .gap_2()
                    .items_center()
                    .child(self.statistics_button(IconName::RefreshCw, "statistics-refresh", "重新扫描项目", cx))
                    .child(self.statistics_button(IconName::Copy, "statistics-copy", "复制目录统计", cx))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(format!("{} 个目录 · {} 个文件 · {} 行", snapshot.directories.len(), snapshot.total.count, snapshot.total.lines,)),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(crate::gui::component::appearance::ui_text_size(11.))
                            .text_color(crate::component::appearance::foreground(MUTED))
                            .child(if self.state.statistics_loading {
                                "正在扫描…".to_owned()
                            } else if snapshot.scanned_at.is_empty() {
                                String::new()
                            } else {
                                format!("更新于 {}", snapshot.scanned_at)
                            }),
                    ),
            )
            .when_some(snapshot.error.clone(), |view, error| {
                view.child(div().px_3().py_2().text_color(crate::component::appearance::foreground(0xc42b35)).child(error))
            })
            .child(
                v_flex()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .border_t_1()
                    .border_color(crate::component::appearance::border(BORDER))
                    .when(snapshot.directories.is_empty(), |view| {
                        view.child(div().p_4().text_color(crate::component::appearance::foreground(MUTED)).child(if self.state.statistics_loading {
                            "正在扫描目录和文件类型…"
                        } else if !DATA_CONTEXT.has_project() {
                            "请先打开项目"
                        } else {
                            "根目录下暂无可统计的文件夹"
                        }))
                    })
                    .child(body)
                    .child(Scrollbar::vertical(&self.state.statistics_scroll)),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px_3()
                    .py_2()
                    .text_size(crate::gui::component::appearance::ui_text_size(11.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child(format!(
                        "仅列出根目录下的文件夹，行数包含所有子目录 · 行数包含空行 · 排除版本控制和构建目录 · 二进制文件不计行数 · 跳过 {} 个不可读项",
                        snapshot.skipped_files
                    )),
            )
            .into_any_element()
    }

    fn statistics_directory(
        &self,
        directory: &DirectoryStatistics,
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let collapsed = self.state.statistics_collapsed.contains(&directory.path);
        let path = directory.path.clone();
        let label = directory_label(directory);
        let tooltip = label.clone();
        h_flex()
            .id(SharedString::from(format!("statistics-directory-{index}")))
            .h(px(32.))
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .px_3()
            .gap_2()
            .items_center()
            .bg(crate::component::appearance::background(0xf0f2f6))
            .border_b_1()
            .border_color(crate::component::appearance::border(BORDER))
            .cursor_pointer()
            .hover(|style| style.bg(crate::component::appearance::background(0xe8edf5)))
            .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
            .child(icon(
                if collapsed {
                    IconName::ChevronRight
                } else {
                    IconName::ChevronDown
                },
                12.,
                MUTED,
            ))
            .child(icon(
                if collapsed {
                    IconName::FolderClosed
                } else {
                    IconName::FolderOpen
                },
                15.,
                MUTED,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(label),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(crate::gui::component::appearance::ui_text_size(11.))
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child(format!(
                        "{} 种类型 · {} 个文件 · {} 行",
                        directory.extensions.len(),
                        directory.total.count,
                        directory.total.lines
                    )),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                if !this.state.statistics_collapsed.remove(&path) {
                    this.state.statistics_collapsed.insert(path.clone());
                }
                cx.notify();
            }))
            .into_any_element()
    }

    fn statistics_type(&self, row: &ExtensionStatistics) -> AnyElement {
        h_flex()
            .h(px(32.))
            .w_full()
            .min_w_0()
            .pl(px(52.))
            .pr_3()
            .gap_2()
            .items_center()
            .text_size(crate::gui::component::appearance::ui_text_size(12.))
            .child(icon(IconName::File, 14., MUTED))
            .child(div().min_w_0().truncate().child(type_summary(row)))
            .into_any_element()
    }

    fn statistics_button(
        &self,
        name: IconName,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = DATA_CONTEXT.has_project() && !self.state.statistics_loading;
        crate::gui::component::controls::icon_button(id, 26., 26.)
            .rounded_sm()
            .when(enabled, |button| button.cursor_pointer().hover(|style| style.bg(crate::component::appearance::background(0xe7e9ed))))
            .when(!enabled, |button| button.opacity(0.45))
            .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
            .child(icon(name, 14., MUTED))
            .on_click(cx.listener(move |this, _, _, cx| {
                if !enabled {
                    return;
                }
                if id == "statistics-refresh" {
                    this.refresh_statistics(cx);
                } else {
                    let snapshot = DATA_CONTEXT.project.statistics();
                    let mut text = format!(
                        "目录统计：{} 个目录 · {} 个文件 · {} 行\n（仅列根目录文件夹，汇总所有子目录；行数包含空行，二进制文件不计行数）\n",
                        snapshot.directories.len(),
                        snapshot.total.count,
                        snapshot.total.lines
                    );
                    for directory in &snapshot.directories {
                        text.push_str(&format!("\n{} · {} 个文件 · {} 行\n", directory_label(directory), directory.total.count, directory.total.lines));
                        for row in &directory.extensions {
                            text.push_str(&format!("  {}\n", type_summary(row)));
                        }
                    }
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            }))
            .into_any_element()
    }
}

fn directory_label(directory: &DirectoryStatistics) -> String {
    directory.path.to_string_lossy().into_owned()
}

fn type_summary(row: &ExtensionStatistics) -> String {
    let extension = if row.extension == "[no extension]" {
        "无扩展名".to_owned()
    } else {
        format!(".{}", row.extension)
    };
    let lines = if row.binary_files == row.count {
        "二进制文件，不计行数".to_owned()
    } else if row.binary_files > 0 {
        format!("{} 行（另有 {} 个二进制文件）", row.lines, row.binary_files)
    } else {
        format!("{} 行", row.lines)
    };
    format!("{extension} · {} 个文件 · {lines}", row.count)
}
