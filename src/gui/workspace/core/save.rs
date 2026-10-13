use super::super::{ProjectFile, Workspace};
use crate::application::APPLICATION;
use crate::application::lifecycle::run_application;
use crate::data_context::DATA_CONTEXT;
use gpui_kit::{Context, Window};
use std::time::Duration;

impl Workspace {
    pub(in crate::gui::workspace) const AUTOSAVE_DELAY: Duration = Duration::from_secs(3);
    pub(in crate::gui::workspace) fn schedule_autosave(
        &mut self,
        file: ProjectFile,
        cx: &mut Context<Self>,
    ) {
        // Replacing a waiting task cancels its timer, never an in-progress write.
        self.autosave_timers.remove(&file);
        if self.closing || self.close_prompt_open {
            return;
        }
        let Some(document) = DATA_CONTEXT
            .documents
            .snapshot(&file.path(DATA_CONTEXT.project_root()))
        else {
            return;
        };
        if !document.dirty() || document.save_error.is_some() {
            return;
        }
        tracing::debug!(file = %file.relative_path().display(), revision = document.revision, delay_ms = Self::AUTOSAVE_DELAY.as_millis(), "document automatic save scheduled");
        let pending_file = file.clone();
        let timer = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Self::AUTOSAVE_DELAY).await;
            let _ = this.update(cx, |this, cx| {
                // Detach the currently executing timer before removing its handle.
                if let Some(timer) = this.autosave_timers.remove(&pending_file) {
                    timer.detach();
                }
                this.save_file(pending_file, true, cx);
            });
        });
        self.autosave_timers.insert(file, timer);
    }

    pub(in crate::gui::workspace) fn resume_autosave(&mut self, cx: &mut Context<Self>) {
        let files: Vec<_> = self.editors.keys().cloned().collect();
        for file in files {
            self.schedule_autosave(file, cx);
        }
    }

    pub(in crate::gui::workspace) fn save_current(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.open_files.is_empty() {
            return;
        }
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
        self.autosave_timers.remove(&file);
        self.notice = Some(format!(
            "正在{}保存 {}…",
            if automatic { "自动" } else { "" },
            file.name()
        ));
        tracing::info!(file = %file.relative_path().display(), revision = document.revision, automatic, "document save requested");

        let documents = &APPLICATION.documents;
        let path = document.path.clone();
        cx.spawn(async move |this, cx| {
            let result = run_application(async move { documents.save(path).await }).await;
            let _ = this.update(cx, |this, cx| {
                this.saving_files.remove(&file);
                match result {
                    Ok(()) => {
                        let dirty = DATA_CONTEXT.documents.snapshot(&file.path(DATA_CONTEXT.project_root())).is_some_and(|document| document.dirty());
                        if dirty {
                            // A newer edit remains dirty and gets its own debounced save.
                            // Keep an existing timer's deadline when the user is still typing.
                            if !this.autosave_timers.contains_key(&file) { this.schedule_autosave(file.clone(), cx); }
                            this.notice = Some(format!("{} 有新修改，等待自动保存", file.name()));
                        } else {
                            this.autosave_timers.remove(&file);
                            this.notice = Some(format!("已{}保存 {}", if automatic { "自动" } else { "" }, file.name()));
                        }
                        this.prune_editors();
                    }
                    Err(error) => {
                        this.autosave_timers.remove(&file);
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
