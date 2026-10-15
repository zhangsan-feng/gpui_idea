use super::super::{ProjectFile, Workspace};
use crate::application::APPLICATION;
use crate::data_context::DATA_CONTEXT;
use crate::{
    application::lifecycle::run_application,
    domain::history::{FileViewState, OpenKind, WindowViewState, WorkspaceSession},
};
use gpui_kit::{Context, Window, WindowBounds};
use std::path::{Path, PathBuf};

impl Workspace {
    pub(in crate::gui::workspace) fn window_view_state(window: &Window) -> WindowViewState {
        let bounds = window.window_bounds();
        let size = bounds.get_bounds().size;
        WindowViewState {
            width: f32::from(size.width),
            height: f32::from(size.height),
            maximized: matches!(bounds, WindowBounds::Maximized(_)),
        }
    }

    pub(in crate::gui::workspace) fn update_window_view(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        let view = Self::window_view_state(window);
        if view != self.window_view && view.width > 0. && view.height > 0. {
            self.window_view = view;
            tracing::debug!(?view, "window size updated for restoration");
            self.schedule_save(cx);
        }
    }

    pub(in crate::gui::workspace) fn project_file(&self, path: &Path) -> ProjectFile {
        ProjectFile(
            path.strip_prefix(DATA_CONTEXT.project_root())
                .unwrap_or(path)
                .to_owned(),
        )
    }

    pub(in crate::gui::workspace) fn workspace_session(
        &self,
        cx: &Context<Self>,
    ) -> WorkspaceSession {
        let root = DATA_CONTEXT.project_root();
        let mut expanded_directories: Vec<_> = self.expanded_directories.iter().cloned().collect();
        expanded_directories.sort();
        let run = DATA_CONTEXT.project.run_configuration();
        WorkspaceSession {
            project_root: root.to_owned(),
            open_files: self.open_files.iter().map(|file| file.path(root)).collect(),
            active_file: (!self.open_files.is_empty()).then(|| self.selected_file.path(root)),
            expanded_directories,
            project_panel_open: self.project_panel_open,
            run_panel_open: self.run_panel_open,
            active_tool: self.active_tool,
            run_command: run.command.clone(),
            run_automatic: run.automatic,
            run_profile: run.selected_profile.clone(),
            run_configurations: run.configurations.clone(),
            selected_run_configuration: run.selected_configuration,
            window_view: Some(self.window_view),
            project_width: self
                .project_resize
                .read(cx)
                .sizes()
                .first()
                .copied()
                .map(f32::from)
                .or_else(|| {
                    DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.project_width)
                }),
            run_height: self
                .run_resize
                .read(cx)
                .sizes()
                .get(1)
                .copied()
                .map(f32::from)
                .or_else(|| {
                    DATA_CONTEXT
                        .startup()
                        .session
                        .as_ref()
                        .and_then(|saved| saved.run_height)
                }),
            file_views: self
                .open_files
                .iter()
                .filter_map(|file| {
                    let path = file.path(root);
                    if let Some(editor) = self.editors.get(file) {
                        let state = editor.read(cx);
                        let cursor = state.cursor_position();
                        let scroll = state.scroll_offset();
                        Some(FileViewState {
                            path,
                            cursor_line: cursor.line,
                            cursor_character: cursor.character,
                            scroll_x: scroll.x.into(),
                            scroll_y: scroll.y.into(),
                        })
                    } else {
                        self.pending_file_views.get(&path).cloned()
                    }
                })
                .collect(),
            revision: self.session_revision,
        }
    }

    pub(in crate::gui::workspace) fn remember_opened(
        &self,
        kind: OpenKind,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) {
        let history = &APPLICATION.history;
        cx.spawn(async move |_, _| {
            if let Err(error) =
                run_application(async move { history.record_opened(kind, path).await }).await
            {
                tracing::warn!(error = %error, "opened path history save failed");
            }
        })
        .detach();
    }

    pub(in crate::gui::workspace) fn close_with_session(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.window_view = Self::window_view_state(window);
        self.closing = true;
        self.persistence_timer.take();
        self.session_revision = self.session_revision.saturating_add(1);
        let session = self.workspace_session(cx);

        let history = &APPLICATION.history;
        cx.spawn_in(window, async move |this, cx| {
            let result = run_application(async move { history.save_session(session).await }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                if let Err(error) = result {
                    tracing::error!(error = %error, "final workspace session save failed");
                    this.notice = Some(format!("工作区状态保存失败：{error:#}"));
                }
                // History failure must not trap the user in a window they asked to close.
                for handle in this
                    .project_windows
                    .values()
                    .copied()
                    .chain(this.run_configuration_window)
                {
                    let _ = handle.update(cx, |_, child, _| child.remove_window());
                }
                window.defer(cx, |window, _| window.remove_window());
            });
        })
        .detach();
        cx.notify();
    }
}
