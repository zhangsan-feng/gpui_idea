use super::CodeSearchWindow;
use crate::application::{
    APPLICATION,
    lifecycle::{run_application, wait_timer},
};
use crate::domain::project::ContentMatch;
use gpui_kit::{Context, Window};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

// Keep IDs unique when a window is reopened while its previous search is running.
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);

impl CodeSearchWindow {
    pub(super) fn search(&mut self, cx: &mut Context<Self>) {
        let request = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        self.request = request;
        let query = self.input.read(cx).value().to_string();
        self.loading = !query.is_empty();
        self.results.clear();
        self.truncated = false;
        self.error = None;
        tracing::info!(
            request,
            query_len = query.len(),
            "code search input changed"
        );
        cx.notify();
        if !self.loading {
            return;
        }
        cx.spawn(async move |this, cx| {
            let result = async {
                wait_timer("code-search-debounce", Duration::from_millis(180)).await?;
                if this
                    .update(cx, |this, _| this.request != request)
                    .unwrap_or(true)
                {
                    return Ok(None);
                }
                run_application(
                    async move { APPLICATION.project.search_content(query, request).await },
                )
                .await
                .map(Some)
            }
            .await;
            let _ = this.update(cx, |this, cx| {
                if this.request != request {
                    tracing::debug!(request, "code search stale completion ignored");
                    return;
                }
                this.loading = false;
                match result {
                    Ok(Some(snapshot)) => {
                        this.results = snapshot.matches;
                        this.truncated = snapshot.truncated;
                        this.error = snapshot.error;
                        tracing::info!(
                            request,
                            matches = this.results.len(),
                            "code search results applied"
                        );
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(request, %error, "code search failed");
                        this.error = Some(format!("搜索失败：{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn open_match(
        &mut self,
        result: ContentMatch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(parent) = self.parent.upgrade() else {
            return;
        };
        crate::event_bus::EVENT_BUS.gui_event.send(
            parent.entity_id(),
            crate::event_bus::gui_event::GuiEvent::Editor(
                crate::gui::workspace::editor::Event::OpenContentMatch(result),
            ),
        );
        let _ = self
            .parent_window
            .update(cx, |_, window, _| window.activate_window());
        window.remove_window();
    }
}
