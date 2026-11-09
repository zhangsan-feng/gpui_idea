use super::super::SettingsWindow;
use crate::{
    application::{APPLICATION, lifecycle::run_application},
    domain::theme::OnlineTheme,
};
use gpui_kit::{Context, PathPromptOptions, Window};

impl SettingsWindow {
    pub(in crate::gui::settings_window) fn select_default_theme(&mut self, cx: &mut Context<Self>) {
        if self.saving || self.system_busy || self.themes.busy {
            return;
        }
        APPLICATION.settings.select_default_theme(&mut self.draft);
        self.themes.message = Some("已选择默认主题，点击“应用”或“确定”后生效。".into());
        self.error = None;
        cx.notify();
    }

    pub(in crate::gui::settings_window) fn select_theme(
        &mut self,
        id: &str,
        cx: &mut Context<Self>,
    ) {
        if self.saving || self.system_busy || self.themes.busy {
            return;
        }
        if APPLICATION.settings.select_theme(&mut self.draft, id) {
            self.themes.message = Some("已选择配色，点击“应用”或“确定”后生效。".into());
            self.error = None;
            cx.notify();
        }
    }

    pub(in crate::gui::settings_window) fn uninstall_theme(
        &mut self,
        source: &str,
        cx: &mut Context<Self>,
    ) {
        if self.saving || self.system_busy || self.themes.busy {
            return;
        }
        if APPLICATION
            .settings
            .uninstall_theme(&mut self.draft, source)
        {
            self.themes.message =
                Some("已移除该主题包及其全部配色，点击“应用”或“确定”后保存；卸载当前主题时恢复默认主题。".into());
            self.error = None;
            cx.notify();
        }
    }

    pub(in crate::gui::settings_window) fn import_theme(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.saving || self.themes.busy {
            return;
        }
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("选择 VS Code 主题 JSON / VSIX".into()),
        });
        self.themes.busy = true;
        self.themes.message = None;
        self.error = None;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = match prompt.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        run_application(
                            async move { APPLICATION.settings.import_themes(path).await },
                        )
                        .await
                        .map(Some)
                    } else {
                        Ok(None)
                    }
                }
                Ok(Err(error)) => Err(error),
                Err(error) => Err(error.into()),
                _ => Ok(None),
            };
            let _ = this.update(cx, |this, cx| {
                this.themes.busy = false;
                match result {
                    Ok(Some(themes)) => this.stage_themes(themes),
                    Ok(None) => {}
                    Err(error) => this.error = Some(format!("导入主题失败：{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui::settings_window) fn search_themes(
        &mut self,
        offset: usize,
        cx: &mut Context<Self>,
    ) {
        self.load_theme_page(offset, true, cx);
    }

    fn load_theme_page(&mut self, offset: usize, fresh: bool, cx: &mut Context<Self>) {
        if self.themes.busy || self.saving {
            return;
        }
        let query = if fresh {
            self.themes.query.read(cx).value().trim().to_owned()
        } else {
            self.themes.searched_query.clone()
        };
        self.themes.busy = true;
        self.themes.message = None;
        self.error = None;
        let pending = query.clone();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = run_application(async move {
                APPLICATION.settings.search_themes(pending, offset).await
            })
            .await;
            let _ = this.update(cx, |this, cx| {
                this.themes.busy = false;
                match result {
                    Ok(result) => {
                        this.themes.results = result.extensions;
                        this.themes.next_offset = result.next_offset;
                        if offset == 0 {
                            this.themes.page_offsets.clear();
                        } else if let Some(index) = this
                            .themes
                            .page_offsets
                            .iter()
                            .position(|previous| *previous == offset)
                        {
                            this.themes.page_offsets.truncate(index);
                        } else {
                            this.themes.page_offsets.push(this.themes.offset);
                        }
                        this.themes.offset = offset;
                        this.themes.searched_query = query;
                        this.themes.loaded = true;
                    }
                    Err(error) => {
                        tracing::warn!(%error, "online theme search failed");
                        this.error = Some(format!("获取在线主题失败：{error:#}"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui::settings_window) fn next_theme_page(&mut self, cx: &mut Context<Self>) {
        if let Some(offset) = self.themes.next_offset {
            self.load_theme_page(offset, false, cx);
        }
    }

    pub(in crate::gui::settings_window) fn previous_theme_page(&mut self, cx: &mut Context<Self>) {
        if let Some(offset) = self.themes.page_offsets.last().copied() {
            self.load_theme_page(offset, false, cx);
        }
    }

    pub(in crate::gui::settings_window) fn download_theme(
        &mut self,
        theme: OnlineTheme,
        cx: &mut Context<Self>,
    ) {
        if self.themes.busy || self.saving {
            return;
        }
        self.themes.busy = true;
        self.themes.message = Some(format!("正在下载 {}…", theme.name));
        self.error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result =
                run_application(async move { APPLICATION.settings.download_themes(theme).await })
                    .await;
            let _ = this.update(cx, |this, cx| {
                this.themes.busy = false;
                this.themes.message = None;
                match result {
                    Ok(themes) => this.stage_themes(themes),
                    Err(error) => this.error = Some(format!("下载主题失败：{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn stage_themes(&mut self, themes: Vec<crate::domain::theme::ImportedTheme>) {
        let count = themes.len();
        APPLICATION.settings.stage_themes(&mut self.draft, themes);
        self.themes.online = false;
        self.themes.message = Some(format!(
            "已载入 {count} 套配色，请选择后点击“应用”或“确定”保存。"
        ));
        self.error = None;
    }
}
