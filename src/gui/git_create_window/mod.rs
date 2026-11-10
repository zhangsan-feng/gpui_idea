mod core;
mod external;
mod ui;

pub(in crate::gui) use external::open;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::{AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CreateKind {
    LocalBranch,
    RemoteBranch,
}

struct GitCreateWindow {
    project_root: PathBuf,
    branch: String,
    remotes: Vec<String>,
    is_repository: bool,
    kind: CreateKind,
    name: Entity<InputState>,
    selected_remote: String,
    busy: bool,
    feedback: Option<(bool, String)>,
    _subscriptions: Vec<Subscription>,
}

impl GitCreateWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let snapshot = crate::data_context::DATA_CONTEXT.project.git();
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("例如 feature/new-feature"));
        let mut subscriptions = Vec::new();
        for input in [&name] {
            subscriptions.push(cx.observe(input, |_, _, cx| cx.notify()));
            subscriptions.push(
                cx.subscribe_in(input, window, |this, _, event, window, cx| match event {
                    InputEvent::Change if !this.busy => {
                        this.feedback = None;
                        cx.notify();
                    }
                    InputEvent::PressEnter { .. } => this.submit(window, cx),
                    _ => {}
                }),
            );
        }
        name.update(cx, |input, cx| input.focus(window, cx));
        Self {
            project_root: crate::data_context::DATA_CONTEXT.project_root().to_owned(),
            branch: snapshot.branch.clone(),
            selected_remote: snapshot
                .remotes
                .iter()
                .find(|r| *r == "origin")
                .or(snapshot.remotes.first())
                .cloned()
                .unwrap_or_default(),
            remotes: snapshot.remotes.clone(),
            is_repository: snapshot.is_repository,
            kind: CreateKind::LocalBranch,
            name,
            busy: false,
            feedback: None,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for GitCreateWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::ParentElement as _;
        crate::gui::component::tool_window::frame("创建 Git 分支", !self.busy, cx)
            .child(self.render_view(cx))
    }
}
