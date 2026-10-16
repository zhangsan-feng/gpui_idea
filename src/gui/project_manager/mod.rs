mod core;
mod create;
mod external;
mod ui;

pub(crate) use external::open;
use gpui_kit::component::input::InputState;
use gpui_kit::{AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum ProjectWindowKind {
    Open,
    Create,
    Manage,
}
impl ProjectWindowKind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Open => "打开新项目",
            Self::Create => "创建新项目",
            Self::Manage => "管理项目",
        }
    }
}

struct ProjectManager {
    mode: ProjectWindowKind,
    directory: Entity<InputState>,
    busy: bool,
    error: Option<String>,
    status: Option<String>,
    subscriptions: Vec<Subscription>,
    creation: create::CreateForm,
}
impl ProjectManager {
    fn new(mode: ProjectWindowKind, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root = crate::data_context::DATA_CONTEXT.project_root();
        let initial = if mode == ProjectWindowKind::Create {
            root.parent().unwrap_or(root)
        } else {
            root
        };
        let mut this = Self {
            mode,
            directory: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("选择项目目录")
                    .default_value(if mode == ProjectWindowKind::Create {
                        initial.join("untitled").to_string_lossy().into_owned()
                    } else {
                        initial.to_string_lossy().into_owned()
                    })
            }),
            busy: false,
            error: None,
            status: None,
            subscriptions: Vec::new(),
            creation: create::CreateForm::new(window, cx),
        };
        for input in [&this.directory] {
            this.subscriptions
                .push(cx.observe(input, |_, _, cx| cx.notify()));
        }
        for input in [
            &this.creation.toolchain,
            &this.creation.module,
            &this.creation.environment,
        ] {
            this.subscriptions
                .push(cx.observe(input, |_, _, cx| cx.notify()));
        }
        if mode == ProjectWindowKind::Create {
            this.inspect_toolchain(window, cx);
        }
        this.start_subscribe(window, cx);
        cx.spawn(async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async {
                crate::application::APPLICATION.history.load().await
            })
            .await;
            let _ = this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.error = Some(format!("无法读取最近项目：{error:#}"));
                }
                cx.notify();
            });
        })
        .detach();
        window.on_window_should_close(cx, {
            let view = cx.entity().downgrade();
            move |_, cx| view.upgrade().is_none_or(|view| !view.read(cx).busy)
        });
        this
    }
    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut changes = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::ProjectManager);
        cx.spawn_in(window, async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();
    }
}
impl Render for ProjectManager {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::{ParentElement as _, Styled as _};
        gpui_kit::component::v_flex()
            .size_full()
            .child(crate::component::operation_title_bar::render(
                self.mode.title(),
                !self.busy,
                cx,
            ))
            .child(
                gpui_kit::div()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_view(cx)),
            )
    }
}
