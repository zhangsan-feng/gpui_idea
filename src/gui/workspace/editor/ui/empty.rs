use super::*;
use crate::gui::workspace::ToolbarAction;
use gpui_kit::component::button::{Button, ButtonVariants as _};

impl Editor {
    pub(super) fn empty_workspace(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_4()
            .child(icon(IconName::FolderOpen, 36., MUTED))
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("尚未打开项目"),
            )
            .child(
                div()
                    .text_color(color(MUTED))
                    .child("打开已有项目或创建项目，开始编辑。"),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        Button::new("empty-workspace-open")
                            .primary()
                            .label("打开项目")
                            .on_click(cx.listener(|this, _, _, _| {
                                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                                    crate::gui::workspace::window_bar::Event::ToolbarAction(
                                        ToolbarAction::OpenProject,
                                    ),
                                ));
                            })),
                    )
                    .child(
                        Button::new("empty-workspace-create")
                            .label("创建项目")
                            .on_click(cx.listener(|this, _, _, _| {
                                this.send(crate::event_bus::gui_event::GuiEvent::WindowBar(
                                    crate::gui::workspace::window_bar::Event::ToolbarAction(
                                        ToolbarAction::CreateProject,
                                    ),
                                ));
                            })),
                    ),
            )
    }
}
