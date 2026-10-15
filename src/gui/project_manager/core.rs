use super::{ProjectManager, ProjectWindowKind};
use crate::application::{APPLICATION, lifecycle::run_application};
use gpui_kit::{Context, PathPromptOptions, Window};
use std::path::PathBuf;

impl ProjectManager {
    pub(super) fn choose_directory(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("选择目录".into()),
        });
        cx.spawn_in(window, async move |this, cx| match prompt.await {
            Ok(Ok(Some(paths))) => {
                if let Some(path) = paths.into_iter().next() {
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.directory.update(cx, |state, cx| {
                            state.set_value(path.to_string_lossy().into_owned(), window, cx)
                        });
                        this.error = None;
                        cx.notify();
                    });
                }
            }
            Ok(Err(error)) => {
                let _ = this.update(cx, |this, cx| {
                    this.error = Some(format!("{error:#}"));
                    cx.notify();
                });
            }
            _ => {}
        })
        .detach();
    }

    pub(super) fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let path = PathBuf::from(self.directory.read(cx).value().as_str());
        if self.mode == ProjectWindowKind::Create {
            let name = self.name.read(cx).value().to_string();
            self.busy = true;
            self.error = None;
            cx.notify();
            cx.spawn_in(window, async move |this, cx| {
                let result = run_application(async move {
                    let root = APPLICATION.projects.create(path, name).await?;
                    // Creation is complete before launch; callers can retry opening if launch fails.
                    APPLICATION
                        .projects
                        .open(root.clone())
                        .await
                        .map_err(|error| {
                            anyhow::anyhow!("项目已创建于 {}，打开失败：{error:#}", root.display())
                        })
                })
                .await;
                let _ = this.update_in(cx, |this, window, cx| {
                    this.busy = false;
                    match result {
                        Ok(()) => window.remove_window(),
                        Err(error) => {
                            this.error = Some(format!("{error:#}"));
                            cx.notify();
                        }
                    }
                });
            })
            .detach();
        } else {
            self.open_project(path, window, cx);
        }
    }

    pub(super) fn open_project(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.projects.open(path).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(()) => window.remove_window(),
                    Err(error) => {
                        this.error = Some(format!("{error:#}"));
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub(super) fn forget_project(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.projects.forget(path).await }).await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(()) => this.status = Some("已移除最近项目记录".into()),
                    Err(error) => this.error = Some(format!("{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
