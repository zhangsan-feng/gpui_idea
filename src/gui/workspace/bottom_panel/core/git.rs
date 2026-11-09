use crate::application::{APPLICATION, lifecycle::run_application};
use crate::domain::git::GitAction;
use crate::gui::workspace::bottom_panel::BottomPanel;
use gpui_kit::{Context, Window};
use std::path::PathBuf;

impl BottomPanel {
    pub(in crate::gui::workspace::bottom_panel) fn show_git_create_branch(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state.git_busy || self.state.git_loading || self.state.git_details_loading {
            return;
        }
        if let Some(handle) = self.state.git_create_window {
            if handle
                .update(cx, |_, window, _| window.activate_window())
                .is_ok()
            {
                return;
            }
        }
        match crate::gui::git_create_window::open(window, cx) {
            Ok(handle) => self.state.git_create_window = Some(handle),
            Err(error) => {
                tracing::warn!(%error, "Git creation window failed to open");
                self.state.git_feedback = Some((false, format!("无法打开创建窗口：{error:#}")));
                cx.notify();
            }
        }
    }
    pub(in crate::gui::workspace::bottom_panel) fn set_git_history(
        &mut self,
        history: bool,
        cx: &mut Context<Self>,
    ) {
        if self.state.git_busy {
            return;
        }
        self.state.git_history_selected = history;
        self.state.git_diff_request += 1;
        self.state.git_diff_key.clear();
        self.state.git_diff_loading = false;
        if history {
            let snapshot = crate::data_context::DATA_CONTEXT.project.git();
            let commit = snapshot
                .commits
                .iter()
                .find(|commit| self.state.selected_git_commit.as_ref() == Some(&commit.hash))
                .or_else(|| snapshot.commits.first());
            if let Some(commit) = commit {
                self.select_git_commit(commit.hash.clone(), cx);
            } else {
                self.state.selected_git_commit = None;
                self.state.git_details_loading = false;
            }
        }
        cx.notify();
    }

    pub(in crate::gui::workspace::bottom_panel) fn git_action(
        &mut self,
        action: GitAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.state.git_busy || self.state.git_loading || self.state.git_details_loading {
            return;
        }
        let committed = matches!(action, GitAction::Commit { .. });
        self.state.git_busy = true;
        self.state.git_commit_running = committed;
        self.state.git_feedback = None;
        if committed {
            self.state.git_commit_result = None;
            self.state.git_show_commit_result = true;
            self.state.git_commit_result_scroll = gpui_kit::UniformListScrollHandle::new();
        }
        self.state.git_diff_request += 1;
        self.state.git_diff_key.clear();
        self.state.git_diff_loading = false;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.project.git_action(action).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.state.git_busy = false;
                this.state.git_commit_running = false;
                let feedback = match result {
                    Ok(message) => {
                        if committed {
                            this.state
                                .git_summary
                                .update(cx, |input, cx| input.set_value("", window, cx));
                            this.state
                                .git_message
                                .update(cx, |input, cx| input.set_value("", window, cx));
                        }
                        (
                            true,
                            if message.is_empty() {
                                "操作完成".into()
                            } else {
                                message
                            },
                        )
                    }
                    Err(error) => {
                        let message = format!("{error:#}");
                        this.send(crate::event_bus::gui_event::GuiEvent::BottomBar(
                            crate::gui::workspace::bottom_bar::Event::Notice(Some(format!(
                                "Git 操作失败：{message}"
                            ))),
                        ));
                        (false, message)
                    }
                };
                tracing::info!(
                    committed,
                    success = feedback.0,
                    output_lines = feedback.1.lines().count(),
                    "Git operation result received for display"
                );
                if committed {
                    this.state.git_commit_result = Some(feedback);
                    this.state.git_show_commit_result = true;
                } else {
                    this.state.git_feedback = Some(feedback);
                }
                if this.state.git_history_selected {
                    this.set_git_history(true, cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui::workspace::bottom_panel) fn select_git_diff(
        &mut self,
        path: PathBuf,
        staged: bool,
        commit: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if self.state.git_busy {
            return;
        }
        let key = format!(
            "{}:{}:{}",
            commit.as_deref().unwrap_or("worktree"),
            staged,
            path.display()
        );
        self.state.git_diff_title = format!(
            "{} · {}",
            path.display(),
            commit
                .as_deref()
                .map(|hash| &hash[..8])
                .unwrap_or(if staged { "已暂存" } else { "工作区" })
        );
        self.state.git_diff_key = key.clone();
        self.state.git_diff_loading = true;
        self.state.git_diff_request += 1;
        let request = self.state.git_diff_request;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = run_application(async move {
                APPLICATION
                    .project
                    .git_diff(key, path, staged, commit)
                    .await
            })
            .await;
            let _ = this.update(cx, |this, cx| {
                if request != this.state.git_diff_request {
                    return;
                }
                this.state.git_diff_loading = false;
                if let Err(error) = result {
                    tracing::warn!(%error, "Git diff failed");
                }
                cx.notify();
            });
        })
        .detach();
    }
}
