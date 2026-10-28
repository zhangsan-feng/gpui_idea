use crate::gui::component::icon::tinted as project_icon;
use crate::gui::component::path::display_path as display_project_path;
use crate::gui::component::project_icon::project_icon as project_entry_icon;
use crate::gui::workspace::file_tree::FileTree;
mod search;
use super::*;
use crate::data_context::DATA_CONTEXT;
use crate::gui::workspace::ui::*;
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

impl FileTree {
    pub(in crate::gui::workspace) fn render_project_panel(
        &self,
        cx: &mut Context<FileTree>,
    ) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let root_menu_view = cx.entity().downgrade();
        let started = std::time::Instant::now();
        let tree = uniform_list(
            "project-tree",
            self.state.tree_rows.len(),
            cx.processor(|this, range: Range<usize>, _, cx| {
                {
                    range
                        .filter_map(|index| this.state.tree_rows.get(index).cloned())
                        .map(|row| this.tree_row(row, cx))
                        .collect::<Vec<_>>()
                }
            }),
        )
        .with_sizing_behavior(ListSizingBehavior::Auto)
        .flex_1()
        .min_h_0()
        .min_w_0()
        .h_full()
        .track_scroll(&self.state.project_tree_scroll);
        let tree = crate::gui::component::measured_element::MeasuredElement::new(
            tree.into_any_element(),
            started,
            {
                let diagnostics = self
                    .peer_window_bar(cx)
                    .read(cx)
                    .state
                    .window_diagnostics
                    .clone();
                move |times, window, cx| diagnostics.borrow_mut().record("tree", times, window, cx)
            },
        );
        v_flex()
            .id("project-panel")
            .track_focus(&self.state.project_tree_focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.state.project_tree_focus.is_focused(window)
                    && this.state.inline_file_edit.is_none()
                    && event.keystroke.key == "delete"
                    && !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.shift
                {
                    cx.stop_propagation();
                    this.delete_project_entries_dialog(
                        this.state.project_selection.iter().cloned().collect(),
                        window,
                        cx,
                    );
                }
            }))
            .size_full()
            .min_h_0()
            .overflow_hidden()
            .bg(palette.panel)
            .child(self.project_header(cx))
            .when(DATA_CONTEXT.has_project(), |panel| {
                panel.child(self.project_root_row(cx))
            })
            .when(!DATA_CONTEXT.has_project(), |panel| {
                panel.child(div().p_4().text_color(palette.muted).child("尚未打开项目"))
            })
            .child(
                h_flex()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .items_stretch()
                    .child(tree)
                    .child(
                        div()
                            .id("project-tree-scrollbar")
                            .relative()
                            .h_full()
                            .w(Scrollbar::width())
                            .flex_shrink_0()
                            .child(
                                Scrollbar::vertical(&self.state.project_tree_scroll)
                                    .viewport_from_layout(),
                            ),
                    )
                    .child(self.project_wheel_handler(cx))
                    .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _, _cx| {
                        if event.button == MouseButton::Right {
                            this.state.project_context_target = None;
                        }
                    }))
                    .context_menu(move |menu, _, cx| {
                        let (path, is_directory, is_link) = root_menu_view
                            .upgrade()
                            .and_then(|view| view.read(cx).state.project_context_target.clone())
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

    fn project_wheel_handler(&self, cx: &Context<FileTree>) -> impl IntoElement {
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

    fn project_header(&self, cx: &mut Context<FileTree>) -> Div {
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
            .when(self.state.project_selection.len() > 1, |header| {
                header.child(
                    div()
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child(format!("已选 {} 项", self.state.project_selection.len())),
                )
            })
            .child(self.small_panel_button(IconName::RefreshCw, "project-refresh", cx))
            .child(self.small_panel_button(IconName::Minus, "project-collapse", cx))
    }

    fn project_root_row(&self, cx: &mut Context<FileTree>) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let menu_view = cx.entity().downgrade();
        let expanded = self.state.expanded_directories.contains(Path::new(""));
        let display_path = display_project_path(DATA_CONTEXT.project_root());
        let tooltip_path = display_path.clone();
        let project_name = DATA_CONTEXT
            .project_root()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| display_path.clone());
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
                if valid_project_drop(drag, Path::new("")) {
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
            .child(project_entry_icon(Path::new(""), true))
            .child(
                div()
                    .min_w_0()
                    .max_w(relative(0.55))
                    .flex_shrink_0()
                    .truncate()
                    .font_weight(FontWeight(600.))
                    .child(project_name),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .pl_1()
                    .text_xs()
                    .text_color(palette.muted)
                    .child(display_path)
                    .truncate(),
            )
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(tooltip_path.clone()).build(window, cx)
            })
            .on_click(cx.listener(|this, _, window, cx| {
                this.state.project_selection.clear();
                this.state.project_tree_focus.focus(window, cx);
                this.toggle_directory(PathBuf::new(), cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, window, cx| {
                    this.state.project_selection.clear();
                    this.state.project_tree_focus.focus(window, cx);
                    crate::event_bus::EVENT_BUS
                        .refresh_gui
                        .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
                }),
            )
            .child(project_drop_hint(Path::new(""), palette))
            .on_drop(cx.listener(|this, drag: &DraggedProjectEntry, window, cx| {
                cx.stop_propagation();
                if valid_project_drop(drag, Path::new("")) {
                    this.move_project_entries(drag.paths.clone(), PathBuf::new(), window, cx);
                } else {
                    this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                        crate::gui::workspace::bottom_bar::Event::Notice(Some(
                            "文件或目录已经位于项目根目录".into(),
                        )),
                    ));
                    crate::event_bus::EVENT_BUS
                        .refresh_gui
                        .notify(&[crate::event_bus::refresh_gui::GuiComponent::FileTree]);
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
        cx: &mut Context<FileTree>,
    ) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        let button = crate::gui::component::controls::icon_button(id, 23., 23.)
            .rounded_sm()
            .hover(|style| style.bg(palette.hover))
            .child(project_icon(name, 14., palette.muted));
        if id == "project-collapse" {
            button.on_click(cx.listener(|this, _, _, _cx| {
                this.send(crate::event_bus::gui_event::GuiEvent::LeftSidebar(
                    crate::gui::workspace::left_sidebar::Event::ToggleProjectPanel,
                ))
            }))
        } else if id == "project-refresh" {
            button.on_click(cx.listener(|this, _, _, cx| this.refresh_project(cx)))
        } else {
            button
        }
    }
}

fn valid_project_drop(drag: &DraggedProjectEntry, target: &Path) -> bool {
    !drag.paths.is_empty()
        && drag.paths.iter().all(|source| {
            source
                .file_name()
                .is_some_and(|name| target.join(name) != *source && !target.starts_with(source))
        })
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
    fn render(
        &mut self,
        _: &mut Window,
        cx: &mut Context<DraggedProjectEntry>,
    ) -> impl IntoElement {
        let palette = ProjectTreePalette::from_theme(cx.theme());
        crate::gui::component::drag_preview::render(
            32.,
            8.,
            palette.drag_border,
            palette.panel,
            palette.foreground,
        )
        .child(project_entry_icon(&self.path, self.is_directory))
        .child(div().truncate().child(if self.paths.len() > 1 {
            format!("{} 等 {} 项", self.name, self.paths.len())
        } else {
            self.name.clone()
        }))
    }
}
