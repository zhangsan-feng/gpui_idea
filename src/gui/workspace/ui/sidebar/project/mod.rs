use super::*;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::component::menu::ContextMenuExt;
use gpui_kit::component::scroll::Scrollbar;
use gpui_kit::component::{ActiveTheme as _, Sizable, Theme};
use std::{
    ops::Range,
    path::{Path, PathBuf},
};

mod inline_edit;
mod menu;
mod tree;
use menu::project_menu;

const PROJECT_HEADER_HEIGHT: f32 = 34.;
const PROJECT_ROOT_ROW_HEIGHT: f32 = 25.;
const PROJECT_TREE_ROW_HEIGHT: f32 = 24.;
const PROJECT_TREE_INDENT: f32 = 16.;
const PROJECT_TREE_LEFT_INSET: f32 = 8.;
const PROJECT_TREE_ICON_SIZE: f32 = 15.;
const PROJECT_TREE_CHEVRON_SIZE: f32 = 13.;
const PROJECT_TREE_STATUS_ICON_SIZE: f32 = 9.;

#[derive(Clone, Copy)]
struct ProjectTreePalette {
    panel: Hsla,
    border: Hsla,
    foreground: Hsla,
    muted: Hsla,
    hover: Hsla,
    selected: Hsla,
    accent: Hsla,
    drop_target: Hsla,
    drag_border: Hsla,
    error: Hsla,
    error_hover: Hsla,
}

impl ProjectTreePalette {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            panel: theme.sidebar,
            border: theme.sidebar_border,
            foreground: theme.sidebar_foreground,
            muted: theme.muted_foreground,
            hover: theme.list_hover,
            selected: theme.list_active,
            accent: theme.primary,
            drop_target: theme.drop_target,
            drag_border: theme.drag_border,
            error: theme.red,
            error_hover: theme.danger_hover,
        }
    }
}

impl Workspace {
    pub(in crate::gui::workspace::ui) fn render_project_panel(
        &self,
        cx: &mut Context<Self>,
    ) -> Div {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let root_menu_view = cx.entity().downgrade();
        v_flex()
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(palette.panel)
            .child(self.project_header(cx))
            .child(self.project_root_row(cx))
            .child(
                h_flex()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .items_stretch()
                    .child(
                        uniform_list(
                            "project-tree",
                            self.tree_rows.len(),
                            cx.processor(|this, range: Range<usize>, _, cx| {
                                range
                                    .filter_map(|index| this.tree_rows.get(index).cloned())
                                    .map(|row| this.tree_row(row, cx))
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .with_sizing_behavior(ListSizingBehavior::Auto)
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .h_full()
                        .track_scroll(&self.project_tree_scroll),
                    )
                    .child(
                        div()
                            .id("project-tree-scrollbar")
                            .relative()
                            .h_full()
                            .w(Scrollbar::width())
                            .flex_shrink_0()
                            .child(
                                Scrollbar::vertical(&self.project_tree_scroll)
                                    .viewport_from_layout(),
                            ),
                    )
                    .child(self.project_wheel_handler(cx))
                    .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _, _| {
                        if event.button == MouseButton::Right {
                            this.project_context_target = None;
                        }
                    }))
                    .context_menu(move |menu, _, cx| {
                        let (path, is_directory, is_link) = root_menu_view
                            .upgrade()
                            .and_then(|view| view.read(cx).project_context_target.clone())
                            .unwrap_or_else(|| (PathBuf::new(), true, false));
                        project_menu(
                            menu,
                            root_menu_view.clone(),
                            path,
                            is_directory,
                            is_link,
                            cx,
                        )
                    }),
            )
    }

    fn project_wheel_handler(&self, cx: &Context<Self>) -> impl IntoElement {
        let workspace = cx.entity().downgrade();
        canvas(
            |bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal),
            move |_, hitbox, window, _| {
                window.on_mouse_event(move |event: &ScrollWheelEvent, phase, window, cx| {
                    if phase == DispatchPhase::Capture && hitbox.should_handle_scroll(window) {
                        if let Some(workspace) = workspace.upgrade() {
                            workspace.update(cx, |this, cx| this.scroll_project_lines(event, cx));
                        }
                    }
                });
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    fn project_header(&self, cx: &mut Context<Self>) -> Div {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        h_flex()
            .h(px(PROJECT_HEADER_HEIGHT))
            .flex_shrink_0()
            .px_2()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(palette.border)
            .bg(palette.panel)
            .text_size(px(13.))
            .text_color(palette.foreground)
            .font_weight(FontWeight(600.))
            .child(div().flex_1().child("Project"))
            .child(self.small_panel_button(IconName::RefreshCw, "project-refresh", cx))
            .child(self.small_panel_button(IconName::Minus, "project-collapse", cx))
    }

    fn project_root_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let menu_view = cx.entity().downgrade();
        let expanded = self.expanded_directories.contains(Path::new(""));
        h_flex()
            .id("project-root")
            .relative()
            .group("project-drop-row")
            .w_full()
            .min_w_0()
            .h(px(PROJECT_ROOT_ROW_HEIGHT))
            .border_1()
            .border_color(rgba(0x00000000))
            .flex_shrink_0()
            .px_2()
            .gap_1()
            .items_center()
            .text_size(px(12.))
            .text_color(palette.foreground)
            .hover(|style| style.bg(palette.hover))
            .drag_over::<DraggedProjectEntry>(move |style, drag, _, _| {
                if valid_project_drop(&drag.path, Path::new("")) {
                    style
                        .bg(palette.drop_target)
                        .border_color(palette.drag_border)
                } else {
                    style.bg(palette.error_hover).border_color(palette.error)
                }
            })
            .child(project_icon(
                if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                },
                PROJECT_TREE_CHEVRON_SIZE,
                palette.muted,
            ))
            .child(project_entry_icon(Path::new(""), true, expanded, palette))
            .child(
                div().font_weight(FontWeight(600.)).child(
                    DATA_CONTEXT
                        .project_root()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                ),
            )
            .child(
                div()
                    .min_w_0()
                    .overflow_hidden()
                    .pl_1()
                    .text_xs()
                    .text_color(palette.muted)
                    .child(DATA_CONTEXT.project_root().to_string_lossy().into_owned())
                    .truncate(),
            )
            .on_click(cx.listener(|this, _, _, cx| this.toggle_directory(PathBuf::new(), cx)))
            .child(project_drop_hint(Path::new(""), palette))
            .on_drop(cx.listener(|this, drag: &DraggedProjectEntry, window, cx| {
                cx.stop_propagation();
                if valid_project_drop(&drag.path, Path::new("")) {
                    this.move_project_entry(drag.path.clone(), PathBuf::new(), window, cx);
                } else {
                    this.notice = Some("文件或目录已经位于项目根目录".into());
                    cx.notify();
                }
            }))
            .context_menu(move |menu, _, cx| {
                project_menu(menu, menu_view.clone(), PathBuf::new(), true, false, cx)
            })
    }

    fn small_panel_button(
        &self,
        name: IconName,
        id: &'static str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let button = div()
            .id(id)
            .w(px(23.))
            .h(px(23.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .hover(|style| style.bg(palette.hover))
            .child(project_icon(name, 14., palette.muted));
        if id == "project-collapse" {
            button.on_click(cx.listener(|this, _, _, cx| this.toggle_project_panel(cx)))
        } else if id == "project-refresh" {
            button.on_click(cx.listener(|this, _, _, cx| this.refresh_project(cx)))
        } else {
            button
        }
    }
}

fn valid_project_drop(source: &Path, target: &Path) -> bool {
    source
        .file_name()
        .is_some_and(|name| target.join(name) != source && !target.starts_with(source))
}

fn project_drop_hint(target: &Path, palette: ProjectTreePalette) -> impl IntoElement {
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "项目根目录".into());
    h_flex()
        .absolute()
        .right_2()
        .top_0()
        .h_full()
        .max_w(px(180.))
        .px_1()
        .gap_1()
        .items_center()
        .bg(palette.panel)
        .text_size(px(11.))
        .text_color(palette.accent)
        .opacity(0.)
        .group_drag_over::<DraggedProjectEntry>("project-drop-row", |style| style.opacity(1.))
        .child(project_icon(
            IconName::Folder,
            PROJECT_TREE_CHEVRON_SIZE,
            palette.accent,
        ))
        .child(div().min_w_0().truncate().child(format!("目标：{name}")))
}

impl Render for DraggedProjectEntry {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        h_flex()
            .h(px(32.))
            .max_w(px(280.))
            .px_2()
            .gap_2()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(palette.drag_border)
            .bg(palette.panel)
            .shadow_md()
            .font_family("Microsoft YaHei UI")
            .text_size(px(12.))
            .text_color(palette.foreground)
            .child(project_entry_icon(
                &self.path,
                self.is_directory,
                false,
                palette,
            ))
            .child(div().truncate().child(self.name.clone()))
    }
}

fn project_icon(name: IconName, size: f32, tint: Hsla) -> Icon {
    Icon::new(name)
        .with_size(Size::Size(px(size)))
        .text_color(tint)
}

fn project_entry_icon(
    path: &Path,
    is_directory: bool,
    _expanded: bool,
    _palette: ProjectTreePalette,
) -> gpui_kit::Img {
    crate::component::project_icon::project_icon(path, is_directory)
}
