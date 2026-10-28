use super::*;
use crate::gui::workspace::ToolbarAction;
use crate::gui::workspace::window_bar::WindowBar;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    menu::{DropdownMenu, PopupMenuItem},
};

impl WindowBar {
    pub(super) fn run_action(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let config = DATA_CONTEXT.project.run_configuration();
        let label = config
            .selected()
            .map(|saved| saved.name.clone())
            .unwrap_or_else(|| "添加配置…".into());
        let selected = config.selected_configuration;
        let busy = config.loading || DATA_CONTEXT.project.run_sessions().starting;
        let entries = config.configurations.clone();
        crate::gui::component::controls::toolbar_button("run-configuration", cx)
            .max_w(px(210.))
            .tooltip("选择或编辑运行配置")
            .child(icon(IconName::CirclePlay, 14., 0xb9bdc5))
            .child(div().min_w_0().truncate().child(label))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5))
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.min_w(px(260.));
                for saved in &entries {
                    let id = saved.id;
                    let name = saved.name.clone();
                    let row_view = view.clone();
                    let select = view.clone();
                    let run_enabled = !busy && !saved.command.trim().is_empty();
                    let running =
                        DATA_CONTEXT
                            .project
                            .run_sessions()
                            .instances
                            .iter()
                            .any(|instance| {
                                let snapshot = DATA_CONTEXT.terminal.snapshot(instance.id);
                                instance.configuration == id
                                    && snapshot.generation > 0
                                    && !snapshot.exited
                            });
                    menu = menu.item(
                        PopupMenuItem::element(move |_, _cx| {
                            let run = row_view.clone();
                            let stop = row_view.clone();
                            h_flex()
                                .id(("run-menu-row", id))
                                .h(px(30.))
                                .w_full()
                                .px(px(4.))
                                .gap(px(5.))
                                .items_center()
                                .rounded(px(3.))
                                .bg(crate::component::appearance::background(if selected == id { 0xd5e2ff } else { 0xffffff }))
                                .child(icon(
                                    IconName::CirclePlay,
                                    14.,
                                    if running { 0x59a869 } else { MUTED },
                                ))
                                .child(div().flex_1().min_w_0().truncate().child(name.clone()))
                                .child(
                                    Button::new(("run-menu-start", id))
                                        .ghost()
                                        .w(px(24.))
                                        .h(px(24.))
                                        .px_0()
                                        .disabled(!run_enabled)
                                        .tooltip("运行此配置")
                                        .child(icon(
                                            IconName::Play,
                                            14.,
                                            if run_enabled { 0x59a869 } else { 0xb8bdc6 },
                                        ))
                                        .on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            window.dispatch_action(
                                                Box::new(gpui_kit::base::actions::Cancel),
                                                cx,
                                            );
                                            let _ = run.update(cx, |this, cx| {
                                                {
                                                    this.toolbar_action(
                                                        ToolbarAction::SelectRunConfiguration(id),
                                                        window,
                                                        cx,
                                                    );
                                                    this.send(crate::event_bus::gui_event::GuiEvent::BottomPanel(crate::gui::workspace::bottom_panel::Event::StartExecution(RunAction::Run)));
                                                }
                                            });
                                        }),
                                )
                                .child(
                                    Button::new(("run-menu-stop", id))
                                        .ghost()
                                        .w(px(24.))
                                        .h(px(24.))
                                        .px_0()
                                        .disabled(!running)
                                        .tooltip("停止此配置的实例")
                                        .child(icon(
                                            IconName::Square,
                                            13.,
                                            if running { 0xf04468 } else { 0xb8bdc6 },
                                        ))
                                        .on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            window.dispatch_action(
                                                Box::new(gpui_kit::base::actions::Cancel),
                                                cx,
                                            );
                                            let _ = stop.update(cx, |this, cx| {
                                                {
                                                    this.toolbar_action(
                                                        ToolbarAction::StopRunConfiguration(id),
                                                        window,
                                                        cx,
                                                    )
                                                }
                                            });
                                        }),
                                )
                        })
                        .on_click(move |_, window, cx| {
                            let _ = select.update(cx, |this, cx| {
                                {
                                    this.toolbar_action(
                                        ToolbarAction::SelectRunConfiguration(id),
                                        window,
                                        cx,
                                    )
                                }
                            });
                        }),
                    );
                }
                menu.separator().item(Self::toolbar_item(
                    "编辑配置…",
                    ToolbarAction::EditRun,
                    true,
                    false,
                    &view,
                ))
            })
    }

    pub(super) fn profile_menu(&self, cx: &Context<WindowBar>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let config = DATA_CONTEXT.project.run_configuration();
        let label = if !config.automatic {
            "custom".into()
        } else if config.selected_profile.is_empty() {
            "default".into()
        } else {
            config.selected_profile.clone()
        };
        let profiles = if self.current_run_profile().is_some() {
            config.profiles.clone()
        } else {
            Vec::new()
        };
        let selected = config.selected_profile.clone();
        crate::gui::component::controls::toolbar_button("run-profile", cx)
            .disabled(profiles.is_empty())
            .tooltip("工具链提供的运行 / 构建配置")
            .child(div().child(label))
            .child(icon(IconName::ChevronDown, 12., 0x9399a5))
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.label("配置");
                for profile in &profiles {
                    menu = menu.item(Self::toolbar_item(
                        profile.name.clone(),
                        ToolbarAction::Profile(profile.name.clone()),
                        true,
                        selected == profile.name,
                        &view,
                    ));
                }
                menu
            })
    }
}
