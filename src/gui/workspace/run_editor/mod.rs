mod core;
mod ui;

use super::Workspace;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::component::input::InputState;
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, Render, Subscription, WeakEntity, Window,
};

pub(super) struct RunConfigurationEditor {
    parent: WeakEntity<Workspace>,
    parent_window: gpui_kit::AnyWindowHandle,
    editing: u64,
    loaded_request: u64,
    was_loading: bool,
    dirty: bool,
    command_dirty: bool,
    name: Entity<InputState>,
    command: Entity<InputState>,
    arguments: Entity<InputState>,
    instances: Entity<InputState>,
    parallel: bool,
    error: Option<String>,
    subscriptions: Vec<Subscription>,
}

impl RunConfigurationEditor {
    pub(super) fn new(
        parent: WeakEntity<Workspace>,
        parent_window: gpui_kit::AnyWindowHandle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let configuration = DATA_CONTEXT.project.run_configuration();
        let selected = configuration.selected();
        let mut this = Self {
            parent,
            parent_window,
            editing: configuration.selected_configuration,
            loaded_request: configuration.request,
            was_loading: configuration.loading,
            dirty: false,
            command_dirty: false,
            name: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(selected.map(|saved| saved.name.clone()).unwrap_or_default())
            }),
            command: cx.new(|cx| {
                InputState::new(window, cx).default_value(
                    selected
                        .map(|saved| saved.command.clone())
                        .unwrap_or_default(),
                )
            }),
            arguments: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("例如：--port 8080 --name \"实例 A\"")
                    .default_value(
                        selected
                            .map(|saved| saved.arguments.clone())
                            .unwrap_or_default(),
                    )
            }),
            instances: cx.new(|cx| {
                InputState::new(window, cx).default_value(
                    selected
                        .map(|saved| saved.instances)
                        .unwrap_or(1)
                        .to_string(),
                )
            }),
            parallel: selected.is_some_and(|saved| saved.allow_parallel),
            error: None,
            subscriptions: Vec::new(),
        };
        for input in [&this.name, &this.command, &this.arguments, &this.instances] {
            this.subscriptions
                .push(cx.observe(input, |_, _, cx| cx.notify()));
            this.subscriptions
                .push(cx.subscribe(input, |this, _, event, cx| {
                    if matches!(event, gpui_kit::component::input::InputEvent::Change) {
                        this.dirty = true;
                        this.error = None;
                        cx.notify();
                    }
                }));
        }
        this.start_subscribe(window, cx);
        this.subscriptions
            .push(cx.subscribe(&this.command, |this, _, event, _| {
                if matches!(event, gpui_kit::component::input::InputEvent::Change) {
                    this.command_dirty = true;
                }
            }));
        this
    }

    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut changes = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::RunConfiguration);
        cx.spawn_in(window, async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this
                    .update_in(cx, |this, window, cx| {
                        let configuration = DATA_CONTEXT.project.run_configuration();
                        if this.editing != configuration.selected_configuration
                            || (!this.dirty
                                && !configuration.loading
                                && (this.was_loading
                                    || this.loaded_request != configuration.request))
                        {
                            this.load_selected(window, cx);
                        } else if !this.command_dirty
                            && !configuration.loading
                            && (this.was_loading || this.loaded_request != configuration.request)
                        {
                            if let Some(saved) = configuration.selected() {
                                this.command.update(cx, |state, cx| {
                                    state.set_value(saved.command.clone(), window, cx)
                                });
                            }
                            this.loaded_request = configuration.request;
                        }
                        this.was_loading = configuration.loading;
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }
}

impl Render for RunConfigurationEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::{ParentElement as _, Styled as _};
        gpui_kit::component::v_flex()
            .size_full()
            .child(crate::component::operation_title_bar::render(
                "运行 / 调试配置",
                true,
                cx,
            ))
            .child(
                gpui_kit::div()
                    .flex_1()
                    .min_h_0()
                    .p_4()
                    .child(self.render_view(cx)),
            )
    }
}
