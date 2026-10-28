use super::*;
use crate::gui::component::path::display_path as display_project_path;
use crate::gui::workspace::{ToolbarAction, window_bar::WindowBar};
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    popover::{Popover, PopoverState},
    scroll::ScrollableElement as _,
};
use std::path::PathBuf;

const HEADER_HEIGHT: f32 = 40.;
const FOOTER_HEIGHT: f32 = 44.;
const PROJECT_ROW_HEIGHT: f32 = 56.;
const PROJECT_ROW_GAP: f32 = 2.;
const PROJECT_LIMIT: usize = 15;

impl WindowBar {
    pub(super) fn project_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let root = DATA_CONTEXT.project_root();
        let name = if DATA_CONTEXT.has_project() {
            root.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            "打开项目".to_owned()
        };
        let initials: String = name
            .split(['_', '-', ' '])
            .filter_map(|part| part.chars().next())
            .take(2)
            .flat_map(char::to_uppercase)
            .collect();
        let trigger = crate::gui::component::controls::toolbar_button("project-menu", cx)
            .max_w(px(190.))
            .tooltip(if DATA_CONTEXT.has_project() {
                display_project_path(root)
            } else {
                "最近项目与项目管理".to_owned()
            })
            .child(
                div()
                    .w(px(20.))
                    .h(px(20.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .bg(crate::component::appearance::background(0x159cab))
                    .text_color(crate::component::appearance::foreground(0xffffff))
                    .text_size(px(11.))
                    .when(DATA_CONTEXT.has_project(), |tile| tile.child(initials))
                    .when(!DATA_CONTEXT.has_project(), |tile| {
                        tile.child(icon(IconName::FolderOpen, 13., 0xffffff))
                    }),
            )
            .child(div().min_w_0().truncate().child(name))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5));
        Popover::new("project-switcher")
            .p_0()
            .trigger(trigger)
            .content(move |_, window, cx| project_menu_content(&view, window, cx))
    }
}

fn project_menu_content(
    view: &WeakEntity<WindowBar>,
    window: &Window,
    cx: &Context<PopoverState>,
) -> Div {
    let history = DATA_CONTEXT.history.snapshot();
    let projects: Vec<_> = history
        .entries
        .iter()
        .filter(|entry| entry.kind == crate::domain::history::OpenKind::Project)
        .map(|entry| entry.path.clone())
        .collect();
    let panel_height = (window.bounds().size.height - px(80.)).min(px(520.));
    v_flex()
        .w(px(328.))
        .min_h_0()
        .overflow_hidden()
        .font_family(crate::component::appearance::ui_font())
        .text_size(px(13.))
        .text_color(crate::component::appearance::foreground(TEXT))
        .on_key_down(|event, window, cx| {
            match event.keystroke.key.as_str() {
                "down" => window.focus_next(cx),
                "up" => window.focus_prev(cx),
                _ => return,
            }
            cx.stop_propagation();
        })
        .child(project_menu_header(projects.len()))
        .child(
            project_list(
                projects,
                panel_height - px(HEADER_HEIGHT + FOOTER_HEIGHT),
                view,
                cx,
            )
            .overflow_y_scrollbar(),
        )
        .child(
            div()
                .h(px(FOOTER_HEIGHT))
                .flex_shrink_0()
                .p_1()
                .border_t_1()
                .border_color(crate::component::appearance::border(0xe5e7eb))
                .child(project_management_button(view, cx)),
        )
}

fn project_menu_header(total: usize) -> Div {
    project_menu_row()
        .h(px(HEADER_HEIGHT))
        .flex_shrink_0()
        .px_3()
        .child(
            div()
                .flex_1()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(crate::component::appearance::foreground(0x64748b))
                .child("最近项目"),
        )
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(crate::component::appearance::background(0xf1f5f9))
                .text_size(px(11.))
                .text_color(crate::component::appearance::foreground(0x64748b))
                .child(total.to_string()),
        )
}

fn project_list(
    projects: Vec<PathBuf>,
    max_height: Pixels,
    view: &WeakEntity<WindowBar>,
    cx: &Context<PopoverState>,
) -> Stateful<Div> {
    let visible_count = projects.len().min(PROJECT_LIMIT);
    let content_height = if visible_count == 0 {
        48.
    } else {
        visible_count as f32 * PROJECT_ROW_HEIGHT
            + visible_count.saturating_sub(1) as f32 * PROJECT_ROW_GAP
    };
    // Scrollable uses a percentage-height inner viewport. Give its outer shell
    // a definite height so an auto-sized popover cannot collapse the list.
    let viewport_height = px(content_height).min(max_height.max(px(48.)));
    tracing::debug!(
        project_count = projects.len(),
        visible_count,
        viewport_height_px = f32::from(viewport_height),
        "project switcher list viewport sized"
    );
    let mut list = v_flex()
        .id("recent-project-list")
        .w_full()
        .min_h_0()
        .h(viewport_height)
        .flex_shrink_0()
        .px_1()
        .gap_y(px(PROJECT_ROW_GAP));
    if projects.is_empty() {
        return list.child(
            project_menu_row()
                .h(px(48.))
                .px_2()
                .child(icon(IconName::FolderOpen, 18., 0x94a3b8))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(crate::component::appearance::foreground(0x7b8494))
                        .child("暂无最近项目"),
                ),
        );
    }
    for (index, path) in projects.into_iter().take(PROJECT_LIMIT).enumerate() {
        let current = DATA_CONTEXT.has_project()
            && crate::domain::history::project_identity(&path)
                == crate::domain::history::project_identity(DATA_CONTEXT.project_root());
        let open_view = view.clone();
        list = list.child(
            Button::new(("recent-project", index))
                .ghost()
                .w_full()
                .h(px(PROJECT_ROW_HEIGHT))
                .flex_shrink_0()
                .px_2()
                .rounded_md()
                .tooltip(display_project_path(&path))
                .child(project_row(&path, current))
                .on_click(cx.listener(move |popover, _, window, cx| {
                    cx.stop_propagation();
                    popover.dismiss(window, cx);
                    let _ = open_view.update(cx, |this, cx| {
                        this.toolbar_action(
                            ToolbarAction::OpenRecentProject(path.clone()),
                            window,
                            cx,
                        )
                    });
                })),
        );
    }
    list
}

fn project_row(path: &std::path::Path, current: bool) -> Div {
    let display_path = display_project_path(path);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| display_path.clone());
    project_menu_row()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(32.))
                .flex_shrink_0()
                .rounded_md()
                .bg(crate::component::appearance::background(0xeff6ff))
                .child(icon(IconName::Folder, 17., 0x3b82f6)),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .min_w_0()
                                .truncate()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(crate::component::appearance::foreground(TEXT))
                                .child(name),
                        )
                        .when(current, |row| {
                            row.child(
                                div()
                                    .flex_shrink_0()
                                    .px_1()
                                    .rounded_sm()
                                    .bg(crate::component::appearance::background(0xeff6ff))
                                    .text_size(px(10.))
                                    .text_color(crate::component::appearance::foreground(0x2563eb))
                                    .child("当前"),
                            )
                        }),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(11.))
                        .text_color(crate::component::appearance::foreground(0x7b8494))
                        .child(display_path),
                ),
        )
}

fn project_management_button(view: &WeakEntity<WindowBar>, cx: &Context<PopoverState>) -> Button {
    let view = view.clone();
    Button::new("manage-projects")
        .ghost()
        .w_full()
        .h(px(36.))
        .px_2()
        .child(
            project_menu_row()
                .child(icon(IconName::Settings, 16., 0x64748b))
                .child(div().flex_1().child("管理项目"))
                .child(icon(IconName::ChevronRight, 13., 0x94a3b8)),
        )
        .on_click(cx.listener(move |popover, _, window, cx| {
            cx.stop_propagation();
            popover.dismiss(window, cx);
            let _ = view.update(cx, |this, cx| {
                this.toolbar_action(ToolbarAction::ManageProjects, window, cx)
            });
        }))
}

fn project_menu_row() -> Div {
    h_flex().w_full().min_w_0().items_center().gap_3()
}
