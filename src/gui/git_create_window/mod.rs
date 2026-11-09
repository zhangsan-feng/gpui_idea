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
    Repository,
}

struct GitCreateWindow {
    project_root: PathBuf,
    branch: String,
    remotes: Vec<String>,
    is_repository: bool,
    kind: CreateKind,
    name: Entity<InputState>,
    organization: Entity<InputState>,
    description: Entity<InputState>,
    remote_name: Entity<InputState>,
    token: Entity<InputState>,
    selected_remote: String,
    private: bool,
    busy: bool,
    feedback: Option<(bool, String)>,
    _subscriptions: Vec<Subscription>,
}

impl GitCreateWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let snapshot = crate::data_context::DATA_CONTEXT.project.git();
        let name = cx.new(|cx| {
            InputState::new(window, cx).placeholder("例如 feature/new-feature 或 my-project")
        });
        let organization = cx
            .new(|cx| InputState::new(window, cx).placeholder("留空创建到当前令牌所属的个人账号"));
        let description = cx.new(|cx| InputState::new(window, cx).placeholder("仓库描述（可选）"));
        let remote_name = cx.new(|cx| {
            InputState::new(window, cx).default_value(
                if snapshot.remotes.iter().any(|r| r == "origin") {
                    "github"
                } else {
                    "origin"
                },
            )
        });
        let token = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("GitHub Personal Access Token")
                .masked(true)
        });
        let mut subscriptions = Vec::new();
        for input in [&name, &organization, &description, &remote_name, &token] {
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
            kind: if snapshot.is_repository {
                CreateKind::LocalBranch
            } else {
                CreateKind::Repository
            },
            name,
            organization,
            description,
            remote_name,
            token,
            private: true,
            busy: false,
            feedback: None,
            _subscriptions: subscriptions,
        }
    }
}

impl Render for GitCreateWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_view(cx)
    }
}
