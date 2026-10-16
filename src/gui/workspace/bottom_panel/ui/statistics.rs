use super::*;
use crate::domain::statistics::ExtensionStatistics;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::tooltip::Tooltip;

const COLUMNS: [&str; 11] = [
    "Extension",
    "Count",
    "Size SUM",
    "Size MIN",
    "Size MAX",
    "Size AVG",
    "Lines",
    "Lines MIN",
    "Lines MAX",
    "Lines AVG",
    "Lines NONEMPTY",
];
impl BottomPanel {
    pub(super) fn render_statistics_panel(&self, cx: &mut Context<BottomPanel>) -> AnyElement {
        let snapshot = DATA_CONTEXT.project.statistics();
        let mut rows = snapshot.extensions.clone();
        rows.sort_by(|a, b| {
            let order = if self.state.statistics_sort == 0 {
                a.extension.cmp(&b.extension)
            } else {
                sort_value(a, self.state.statistics_sort)
                    .total_cmp(&sort_value(b, self.state.statistics_sort))
            };
            if self.state.statistics_descending {
                order.reverse()
            } else {
                order
            }
        });
        let header = h_flex()
            .h(px(50.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(color(BORDER))
            .bg(color(PANEL_BG))
            .children(COLUMNS.into_iter().enumerate().map(|(index, label)| {
                let summary = if index == 1 {
                    snapshot.total.count.to_string()
                } else if index == 0 {
                    String::new()
                } else {
                    values(&snapshot.total)[index].clone()
                };
                let selected = self.state.statistics_sort == index;
                column(
                    div().id(SharedString::from(format!("statistics-column-{index}"))),
                    index,
                )
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.))
                .cursor_pointer()
                .hover(|style| style.bg(color(0xe9ecf2)))
                .child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(label)
                        .when(selected, |view| {
                            view.child(icon(
                                if self.state.statistics_descending {
                                    IconName::ChevronDown
                                } else {
                                    IconName::ChevronUp
                                },
                                10.,
                                MUTED,
                            ))
                        }),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(color(MUTED))
                        .child(summary),
                )
                .on_click(cx.listener(move |this, _, _, _cx| {
                    if this.state.statistics_sort == index {
                        this.state.statistics_descending = !this.state.statistics_descending;
                    } else {
                        this.state.statistics_sort = index;
                        this.state.statistics_descending = false;
                    }
                    crate::event_bus::EVENT_BUS
                        .refresh_gui
                        .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomPanel]);
                }))
                .into_any_element()
            }));
        let extension_count = rows.len();
        rows.push(snapshot.total.clone());
        let body = uniform_list(
            "statistics-table-rows",
            rows.len(),
            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                {
                    range
                        .filter_map(|index| {
                            rows.get(index)
                                .map(|row| this.statistics_row(row, index == extension_count, cx))
                        })
                        .collect::<Vec<_>>()
                }
            }),
        )
        .with_sizing_behavior(ListSizingBehavior::Auto)
        .flex_1()
        .min_h_0()
        .w_full()
        .track_scroll(&self.state.statistics_scroll);
        v_flex()
            .size_full()
            .min_h_0()
            .border_t_1()
            .border_color(color(BORDER))
            .bg(color(PANEL_BG))
            .child(self.tool_header(
                ToolWindow::Statistics,
                "Statistic",
                IconName::ChartNoAxesCombined,
                cx,
            ))
            .child(
                h_flex()
                    .h(px(32.))
                    .px_2()
                    .gap_1()
                    .items_center()
                    .child(self.statistics_button(
                        IconName::RefreshCw,
                        "statistics-refresh",
                        "重新扫描项目",
                        cx,
                    ))
                    .child(self.statistics_button(
                        IconName::Copy,
                        "statistics-copy",
                        "复制统计表",
                        cx,
                    ))
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_color(color(MUTED))
                            .child(if self.state.statistics_loading {
                                "正在扫描项目…".into()
                            } else {
                                format!("{} files · {}", snapshot.total.count, snapshot.scanned_at)
                            }),
                    ),
            )
            .child(
                h_flex()
                    .h(px(30.))
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(color(BORDER))
                    .child(
                        h_flex()
                            .h_full()
                            .px_3()
                            .gap_1()
                            .items_center()
                            .border_b_2()
                            .border_color(color(ACCENT))
                            .child(icon(IconName::ListTree, 14., MUTED))
                            .child("Overview"),
                    ),
            )
            .child(
                div()
                    .id("statistics-table-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_x_scroll()
                    .m_2()
                    .border_1()
                    .border_color(color(BORDER))
                    .child(
                        v_flex()
                            .min_w(px(1180.))
                            .w_full()
                            .h_full()
                            .min_h_0()
                            .child(header)
                            .when_some(snapshot.error.clone(), |table, error| {
                                table.child(div().p_3().text_color(color(0xc42b35)).child(error))
                            })
                            .child(
                                v_flex()
                                    .relative()
                                    .flex_1()
                                    .min_h_0()
                                    .w_full()
                                    .overflow_hidden()
                                    .child(body)
                                    .child(Scrollbar::vertical(&self.state.statistics_scroll)),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .h(px(24.))
                    .flex_shrink_0()
                    .px_3()
                    .items_center()
                    .text_size(px(11.))
                    .text_color(color(MUTED))
                    .child("项目实际文件类型 · 排除版本控制与构建目录 · 二进制文件仅统计数量和大小 · Lines NONEMPTY 统计非空行")
                    .child(div().flex_1())
                    .child(format!(
                        "跳过 {} 个二进制或不可读文件",
                        snapshot.skipped_files
                    )),
            )
            .into_any_element()
    }
    fn statistics_row(
        &self,
        row: &ExtensionStatistics,
        total: bool,
        cx: &mut Context<BottomPanel>,
    ) -> AnyElement {
        let selected = self.state.statistics_selected.as_ref() == Some(&row.extension);
        let extension = row.extension.clone();
        let items = values(row);
        h_flex()
            .id(SharedString::from(format!("statistics-row-{extension}")))
            .h(px(22.))
            .w_full()
            .items_center()
            .border_b_1()
            .border_color(color(0xe8eaee))
            .bg(color(if selected { 0xdce8ff } else { PANEL_BG }))
            .hover(|style| style.bg(color(if selected { 0xdce8ff } else { 0xedf0f5 })))
            .children(items.into_iter().enumerate().map(|(index, text)| {
                column(div(), index)
                    .h_full()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_1()
                    .text_size(px(12.))
                    .when(total, |cell| cell.font_weight(FontWeight(600.)))
                    .when(index == 0, |cell| {
                        cell.child(icon(IconName::List, 12., MUTED))
                    })
                    .when((2..=5).contains(&index), |cell| {
                        cell.child(icon(IconName::Save, 12., MUTED))
                    })
                    .when(index >= 6 && index <= 9, |cell| {
                        cell.child(icon(IconName::Eye, 12., MUTED))
                    })
                    .child(div().truncate().child(text))
                    .into_any_element()
            }))
            .on_click(cx.listener(move |this, _, _, _cx| {
                this.state.statistics_selected = Some(extension.clone());
                crate::event_bus::EVENT_BUS
                    .refresh_gui
                    .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomPanel]);
            }))
            .into_any_element()
    }
    fn statistics_button(
        &self,
        name: IconName,
        id: &'static str,
        label: &'static str,
        cx: &mut Context<BottomPanel>,
    ) -> AnyElement {
        div()
            .id(id)
            .w(px(26.))
            .h(px(26.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .cursor_pointer()
            .hover(|style| style.bg(color(0xe7e9ed)))
            .tooltip(move |window, cx| Tooltip::new(label).build(window, cx))
            .child(icon(name, 14., MUTED))
            .on_click(cx.listener(move |this, _, _, cx| {
                if id == "statistics-refresh" {
                    this.refresh_statistics(cx);
                } else {
                    let snapshot = DATA_CONTEXT.project.statistics();
                    let text = std::iter::once(COLUMNS.join("\t"))
                        .chain(
                            snapshot
                                .extensions
                                .iter()
                                .chain(std::iter::once(&snapshot.total))
                                .map(|row| values(row).join("\t")),
                        )
                        .collect::<Vec<_>>()
                        .join("\n");
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            }))
            .into_any_element()
    }
}
fn column<E: Styled>(view: E, index: usize) -> E {
    let view = view
        .flex_shrink_0()
        .border_r_1()
        .border_color(color(0xe8eaee));
    if index == 0 {
        view.w(px(220.))
    } else if index == 1 {
        view.w(px(72.))
    } else {
        view.flex_1().min_w(px(98.))
    }
}
fn values(row: &ExtensionStatistics) -> Vec<String> {
    vec![
        if row.extension == "Total" {
            "Total:  (TOTAL: files)".into()
        } else {
            format!(
                "{}  ({} files)",
                row.extension,
                row.extension.to_uppercase()
            )
        },
        format!("{}x", row.count),
        bytes(row.bytes as f64),
        bytes(row.min_bytes as f64),
        bytes(row.max_bytes as f64),
        bytes(row.average_bytes()),
        row.lines.to_string(),
        row.min_lines.to_string(),
        row.max_lines.to_string(),
        format!("{:.1}", row.average_lines()),
        row.code_lines.to_string(),
    ]
}
fn bytes(value: f64) -> String {
    if value >= 1024. * 1024. {
        format!("{:.1} MB", value / 1024. / 1024.)
    } else if value >= 1024. {
        format!("{:.1} kB", value / 1024.)
    } else {
        format!("{value:.0} B")
    }
}
fn sort_value(row: &ExtensionStatistics, index: usize) -> f64 {
    match index {
        1 => row.count as f64,
        2 => row.bytes as f64,
        3 => row.min_bytes as f64,
        4 => row.max_bytes as f64,
        5 => row.average_bytes(),
        6 => row.lines as f64,
        7 => row.min_lines as f64,
        8 => row.max_lines as f64,
        9 => row.average_lines(),
        _ => row.code_lines as f64,
    }
}
