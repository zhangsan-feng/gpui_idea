use super::*;

impl SettingsWindow {
    pub(super) fn select_page(&mut self, page: SettingsPage, cx: &mut Context<Self>) {
        if self.page == page {
            return;
        }
        self.page = page;
        crate::application::APPLICATION
            .history
            .remember_settings_page(page);
        cx.notify();
        cx.spawn(async move |this, cx| {
            if let Err(error) = crate::application::lifecycle::run_application(async {
                crate::application::APPLICATION
                    .history
                    .persist_settings_page()
                    .await
            })
            .await
            {
                tracing::warn!(%error, "settings window page history save failed");
                let _ = this.update(cx, |this, cx| {
                    this.error = Some(format!("无法保存配置窗口历史：{error:#}"));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub(in crate::gui) fn apply(
        &mut self,
        close: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.saving || self.system_busy || self.themes.busy() {
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
                        crate::component::appearance::apply(cx);
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
