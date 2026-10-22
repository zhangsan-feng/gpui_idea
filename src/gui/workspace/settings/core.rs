use super::*;

impl WorkspaceSettings {
    pub(super) fn apply(&mut self, close: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.saving || self.system_busy {
            return;
        }
        if self.draft == self.saved {
            if close {
                window.remove_window();
            }
            return;
        }
        self.saving = true;
        self.error = None;
        let mut value = self.draft.clone();
        value.lsp_proxy.url = value.lsp_proxy.url.trim().to_owned();
        let pending = value.clone();
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async move {
                crate::application::APPLICATION.settings.save(pending).await
            })
            .await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.saving = false;
                match result {
                    Ok(()) => {
                        this.draft = value.clone();
                        this.saved = value;
                        if close {
                            window.remove_window();
                        }
                    }
                    Err(error) => {
                        tracing::error!(%error, "settings save failed");
                        this.error = Some(format!("保存设置失败：{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}
