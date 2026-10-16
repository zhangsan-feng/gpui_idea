use crate::gui::workspace::file_tree::FileTree;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    data_context::DATA_CONTEXT,
};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{Context, ParentElement as _, Window};
use std::time::Duration;

impl FileTree {
    pub(in crate::gui::workspace) fn search_project(&mut self, cx: &mut Context<FileTree>) {
        self.state.search_request += 1;
        let request = self.state.search_request;
        let query = self.state.search_input.read(cx).value().to_string();
        self.state.search_loading = true;
        self.state.search_results.clear();
        self.state.search_error = None;
        if query.is_empty() {
            self.state.search_loading = false;
            crate::event_bus::EVENT_BUS
                .refresh_gui
                .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
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
                    if this.state.search_request == request {
                        this.state.search_loading = false;
                        this.state.search_error = Some(format!("搜索定时任务失败：{error:#}"));
                        crate::event_bus::EVENT_BUS
                            .refresh_gui
                            .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
                        cx.notify();
                    }
                });
                return;
            }
            if this
                .update(cx, |this, _cx| this.state.search_request != request)
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
                if this.state.search_request != request {
                    return;
                }
                this.state.search_loading = false;
                let snapshot = DATA_CONTEXT.project.content_search();
                if snapshot.request == request {
                    this.state.search_results = snapshot.matches.clone();
                    this.state.search_error = snapshot.error.clone();
                }
                if let Err(error) = result {
                    this.state.search_error = Some(format!("{error:#}"));
                    tracing::warn!(%error, "toolbar content search failed");
                }
                crate::event_bus::EVENT_BUS
                    .refresh_gui
                    .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
                cx.notify();
            });
        })
        .detach();
        crate::event_bus::EVENT_BUS
            .refresh_gui
            .notify(&[crate::event_bus::refresh_gui::GuiComponent::BottomBar]);
        cx.notify();
    }

    pub(in crate::gui::workspace) fn open_file_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<FileTree>,
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
        self.state
            .search_input
            .update(cx, |state, cx| state.focus(window, cx));
    }
}
