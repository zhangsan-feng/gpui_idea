use super::super::{ProjectFile, Workspace};
use crate::application::APPLICATION;
use crate::application::lifecycle::run_application;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::{Context, Window};
use std::collections::HashSet;
use std::time::Duration;

impl Workspace {
    pub(in crate::gui::workspace) const AUTOSAVE_DELAY: Duration = Duration::from_secs(3);

    pub(in crate::gui::workspace) fn schedule_save(&mut self, cx: &mut Context<Self>) {
        if !self.session_ready || self.closing || self.close_prompt_open {
            return;
        }
        self.session_revision = self.session_revision.saturating_add(1);
        self.persistence_timer.take();
        tracing::debug!(
            revision = self.session_revision,
            delay_ms = Self::AUTOSAVE_DELAY.as_millis(),
            "workspace and document save scheduled"
        );
        let timer = cx.spawn(async move |this, cx| {
            if let Err(error) = crate::application::lifecycle::wait_timer(
                "workspace-autosave",
                Self::AUTOSAVE_DELAY,
            )
            .await
            {
                tracing::warn!(%error, "workspace autosave timer stopped");
                return;
            }
            let Ok((session, dirty_files)) = this.update(cx, |this, cx| {
                if let Some(timer) = this.persistence_timer.take() {
                    timer.detach();
                }
                let dirty_files = DATA_CONTEXT
                    .documents
                    .dirty_documents()
                    .iter()
                    .map(|document| this.project_file(&document.path))
                    .collect::<HashSet<_>>();
                for file in &dirty_files {
                    this.save_file(file.clone(), true, cx);
                }
                (this.workspace_session(cx), dirty_files.len())
            }) else {
                return;
            };
            tracing::debug!(
                revision = session.revision,
                dirty_files,
                "workspace and document save timer fired"
            );
            let history = &APPLICATION.history;
            if let Err(error) =
                run_application(async move { history.save_session(session).await }).await
            {
                tracing::error!(error = %error, "workspace session save failed");
                let _ = this.update(cx, |this, cx| {
                    this.notice = Some(format!("工作区状态保存失败：{error:#}"));
                    cx.notify();
                });
            }
        });
        self.persistence_timer = Some(timer);
    }

    pub(in crate::gui::workspace) fn resume_autosave(&mut self, cx: &mut Context<Self>) {
        self.schedule_save(cx);
    }

    pub(in crate::gui::workspace) fn save_current(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_files.is_empty() {
            return;
        }
        self.schedule_save(cx);
        self.save_file(self.selected_file.clone(), false, cx);
    }

    fn save_file(&mut self, file: ProjectFile, automatic: bool, cx: &mut Context<Self>) {
        if self.closing || self.close_prompt_open {
            return;
        }
        let Some(document) = DATA_CONTEXT
            .documents
            .snapshot(&file.path(DATA_CONTEXT.project_root()))
        else {
            return;
        };
        if !document.dirty()
            || document.saving
            || (automatic && document.save_error.is_some())
            || !self.saving_files.insert(file.clone())
        {
            return;
        }
        self.notice = Some(format!(
            "正在{}保存 {}…",
            if automatic { "自动" } else { "" },
            file.name()
        ));
        tracing::info!(file = %file.relative_path().display(), revision = document.revision, automatic, "document save requested");

        let path = document.path.clone();
        cx.spawn(async move |this, cx| {
            let result = run_application(async move { crate::application::APPLICATION.documents.save(path).await }).await;
            let _ = this.update(cx, |this, cx| {
                this.saving_files.remove(&file);
                match result {
                    Ok(()) => {
                        let dirty = DATA_CONTEXT.documents.snapshot(&file.path(DATA_CONTEXT.project_root())).is_some_and(|document| document.dirty());
                        if dirty {
                            // Preserve a timer already scheduled by newer edits.
                            if this.persistence_timer.is_none() {
                                this.schedule_save(cx);
                            }
                            this.notice = Some(format!("{} 有新修改，等待自动保存", file.name()));
                        } else {
                            this.notice = Some(format!("已{}保存 {}", if automatic { "自动" } else { "" }, file.name()));
                        }
                        this.prune_editors();
                    }
                    Err(error) => {
                        tracing::error!(file = %file.relative_path().display(), automatic, error = %error, "document automatic saving paused after save failure");
                        this.notice = Some(format!("{} 保存失败：{error:#}；Ctrl+S 可重试", file.name()));
                    }
                }
                cx.notify();
            });
        }).detach();
        cx.notify();
    }
}
