use super::super::Workspace;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{Context, ParentElement as _, Window};
use std::time::Duration;

impl Workspace {
    pub(in crate::gui::workspace) fn search_project(&mut self, cx: &mut Context<Self>) {
        self.search_request += 1;
        let request = self.search_request;
        let query = self.search_input.read(cx).value().to_string();
        self.search_loading = true;
        self.search_results.clear();
        self.search_error = None;
        if query.is_empty() {
            self.search_loading = false;
            cx.notify();
            return;
        }
        cx.spawn(async move |this, cx| {
            if let Err(error) = crate::application::lifecycle::wait_timer(
                "file-search-debounce",
                Duration::from_millis(180),
            )
            .await
            {
                let _ = this.update(cx, |this, cx| {
                    if this.search_request == request {
                        this.search_loading = false;
                        this.search_error = Some(format!("搜索定时任务失败：{error:#}"));
                        cx.notify();
                    }
                });
                return;
            }
            if this
                .update(cx, |this, _| this.search_request != request)
                .unwrap_or(true)
            {
                return;
            }
            let result =
                run_application(
                    async move { APPLICATION.project.search_content(query, request).await },
                )
                .await;
            let _ = this.update(cx, |this, cx| {
                if this.search_request != request {
                    return;
                }
                this.search_loading = false;
                let snapshot = DATA_CONTEXT.project.content_search();
                if snapshot.request == request {
                    this.search_results = snapshot.matches.clone();
                    this.search_error = snapshot.error.clone();
                }
                if let Err(error) = result {
                    this.search_error = Some(format!("{error:#}"));
                    tracing::warn!(%error, "toolbar content search failed");
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    pub(in crate::gui::workspace) fn open_file_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if window.has_active_dialog(cx) {
            return;
        }
        self.search_project(cx);
        let view = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            let Some(view) = view.upgrade() else {
                return dialog;
            };
            dialog
                .title("代码内容搜索")
                .w(gpui_kit::px(780.))
                .child(view.read(cx).render_file_search(view.downgrade(), cx))
        });
        self.search_input
            .update(cx, |state, cx| state.focus(window, cx));
    }

    pub(in crate::gui::workspace) fn open_content_match(
        &mut self,
        result: crate::domain::project::ContentMatch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use gpui_kit::component::input::{Position, RopeExt as _};
        let file = super::super::ProjectFile(result.path);
        let path = file.path(DATA_CONTEXT.project_root());
        self.pending_search_positions.insert(
            path,
            Position::new(
                result.line.saturating_sub(1),
                result.column.saturating_sub(1),
            ),
        );
        self.select_file(file.clone(), window, cx);
        if let Some(editor) = self.editors.get(&file) {
            self.pending_search_positions
                .remove(&file.path(DATA_CONTEXT.project_root()));
            editor.update(cx, |state, cx| {
                let position = Position::new(
                    result
                        .line
                        .saturating_sub(1)
                        .min(state.text().lines_len().saturating_sub(1) as u32),
                    result.column.saturating_sub(1),
                );
                state.set_cursor_position(position, window, cx);
            });
        }
    }
}
