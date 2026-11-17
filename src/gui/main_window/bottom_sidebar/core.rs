use super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use std::time::{Duration, Instant};

impl BottomSidebar {
    pub(super) fn refresh_modified_time(&mut self, cx: &mut Context<Self>) {
        let editor = self.peer_editor(cx);
        let state = &editor.read(cx).state;
        let path = (!state.open_files.is_empty())
            .then(|| state.selected_file.path(DATA_CONTEXT.project_root()));
        if self.state.modified_file != path {
            self.modified_task = None;
            self.state.modified_file = path.clone();
            self.state.modified_time = None;
            self.state.modified_checked = None;
        }
        let Some(path) = path else { return };
        if self.modified_task.is_some()
            || self
                .state
                .modified_checked
                .is_some_and(|checked| checked.elapsed() < Duration::from_secs(1))
        {
            return;
        }
        self.state.modified_checked = Some(Instant::now());
        self.modified_task = Some(cx.spawn(async move |this, cx| {
            let requested = path.clone();
            let result = run_application(async move {
                APPLICATION.documents.last_modified(requested).await
            })
            .await;
            let modified = match result {
                Ok(modified) => modified,
                Err(error) => {
                    tracing::debug!(path = %path.display(), %error, "status bar file modification time unavailable");
                    None
                }
            };
            let _ = this.update(cx, |this, cx| {
                if this.state.modified_file.as_ref() != Some(&path) {
                    return;
                }
                this.modified_task = None;
                if this.state.modified_time != modified {
                    this.state.modified_time = modified;
                    cx.notify();
                }
            });
        }));
    }
}
