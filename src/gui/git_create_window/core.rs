use super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use crate::domain::git::GitCreateRequest;

impl GitCreateWindow {
    pub(super) fn select_kind(
        &mut self,
        kind: CreateKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy || self.kind == kind {
            return;
        }
        self.kind = kind;
        self.feedback = None;
        self.name.update(cx, |state, cx| {
            state.set_value("", window, cx);
            state.focus(window, cx);
        });
        cx.notify();
    }

    pub(super) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.feedback.as_ref().is_some_and(|(ok, _)| *ok) {
            return;
        }
        let name = self.name.read(cx).value().trim().to_owned();
        if name.is_empty() {
            self.feedback = Some((false, "请输入名称".into()));
            self.name.update(cx, |input, cx| input.focus(window, cx));
            cx.notify();
            return;
        }
        let request = match self.kind {
            CreateKind::LocalBranch => GitCreateRequest::Branch { name, remote: None },
            CreateKind::RemoteBranch => {
                if self.selected_remote.is_empty() {
                    self.feedback = Some((false, "尚未配置远端，请先创建或关联远程仓库".into()));
                    cx.notify();
                    return;
                }
                GitCreateRequest::Branch {
                    name,
                    remote: Some(self.selected_remote.clone()),
                }
            }
            CreateKind::Repository => GitCreateRequest::Repository {
                name,
                organization: self.organization.read(cx).value().trim().to_owned(),
                description: self.description.read(cx).value().trim().to_owned(),
                private: self.private,
                remote: self.remote_name.read(cx).value().trim().to_owned(),
                token: self.token.read(cx).value().trim().to_owned(),
            },
        };
        let root = self.project_root.clone();
        self.busy = true;
        self.feedback = None;
        if self.kind == CreateKind::Repository {
            self.token
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.git_create(root, request).await })
                    .await;
            let _ = this.update_in(cx, |this, _window, cx| {
                this.busy = false;
                this.feedback = Some(match result {
                    Ok(message) => {
                        let snapshot = crate::data_context::DATA_CONTEXT.project.git();
                        this.is_repository = snapshot.is_repository;
                        this.branch = snapshot.branch.clone();
                        this.remotes = snapshot.remotes.clone();
                        this.selected_remote = this.remotes.first().cloned().unwrap_or_default();
                        (true, message)
                    }
                    Err(error) => (false, format!("{error:#}")),
                });
                tracing::info!(
                    success = this.feedback.as_ref().is_some_and(|(ok, _)| *ok),
                    "Git creation result displayed"
                );
                cx.notify();
            });
        })
        .detach();
    }
}
