use super::*;
use crate::application::{APPLICATION, lifecycle::run_application};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

impl BottomSidebar {
    pub(super) fn refresh_modified_time(&mut self, cx: &mut Context<Self>) {
        let editor = self.peer_editor(cx);
        let state = &editor.read(cx).state;
        let root = DATA_CONTEXT.project_root();
        let path = (!state.open_files.is_empty()).then(|| state.selected_file.path(root));
        if self.state.modified_file != path {
            let open_paths: HashSet<_> = state
                .open_files
                .iter()
                .map(|file| file.path(root))
                .collect();
            self.state
                .modified_times
                .retain(|path, _| open_paths.contains(path));
            tracing::debug!(
                file = ?path,
                cached = path.as_ref().is_some_and(|path| self.state.modified_times.contains_key(path)),
                "status bar active file changed"
            );
            self.modified_task = None;
            self.state.modified_file = path.clone();
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
            let _ = this.update(cx, |this, cx| {
                if this.state.modified_file.as_ref() != Some(&path) {
                    return;
                }
                this.modified_task = None;
                let modified = match result {
                    Ok(modified) => modified,
                    Err(error) => {
                        tracing::debug!(path = %path.display(), %error, "status bar file modification time unavailable");
                        return;
                    }
                };
                let previous = match modified {
                    Some(time) => this.state.modified_times.insert(path.clone(), time),
                    None => this.state.modified_times.remove(&path),
                };
                if previous != modified {
                    tracing::debug!(path = %path.display(), ?modified, "status bar file modification time updated");
                    cx.notify();
                }
            });
        }));
    }
}
