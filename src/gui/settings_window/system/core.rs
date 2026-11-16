use super::super::*;
use crate::domain::settings::ContextMenuAction;

impl SettingsWindow {
    pub(in crate::gui) fn manage_context_menu(
        &mut self,
        action: ContextMenuAction,
        cx: &mut Context<Self>,
    ) {
        if self.system_busy || self.saving {
            return;
        }
        self.system_busy = true;
        self.system_message = None;
        self.system_error = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async move {
                crate::application::APPLICATION
                    .settings
                    .context_menu(action)
                    .await
            })
            .await;
            let associations = crate::application::lifecycle::run_application(async {
                crate::application::APPLICATION
                    .settings
                    .file_associations(crate::domain::settings::FileAssociationAction::Refresh)
                    .await
            })
            .await;
            let _ = this.update(cx, |this, cx| {
                this.system_busy = false;
                match associations {
                    Ok(status) => {
                        this.association_status = Some(status);
                        this.association_error = None;
                    }
                    Err(error) => {
                        this.association_status = None;
                        this.association_error = Some(format!("读取文件关联失败：{error:#}"));
                    }
                }
                match result {
                    Ok(status) => {
                        this.system_status = Some(status);
                        this.system_message = match action {
                            ContextMenuAction::Refresh => None,
                            ContextMenuAction::Install => {
                                Some("已添加到 Windows 右键菜单。".into())
                            }
                            ContextMenuAction::Remove => Some("已移除 Windows 右键菜单。".into()),
                        };
                    }
                    Err(error) => {
                        this.system_status = None;
                        this.system_error =
                            Some(format!("操作失败：{error:#}。请刷新状态后重试。"));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui) fn configure_file_associations(
        &mut self,
        open_settings: bool,
        cx: &mut Context<Self>,
    ) {
        if self.system_busy || self.saving {
            return;
        }
        self.system_busy = true;
        self.association_message = None;
        self.association_error = None;
        let input = self.association_input.read(cx).value().to_string();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async move {
                crate::application::APPLICATION.settings.configure_file_associations(input, open_settings).await
            }).await;
            let _ = this.update(cx, |this, cx| {
                this.system_busy = false;
                match result {
                    Ok(status) => {
                        this.association_status = Some(status);
                        this.association_message = Some(if open_settings {
                            "已注册文件类型并打开 Windows 设置，请选择由 GPUI Editor 默认打开的文件类型。"
                        } else {
                            "已添加自定义文件类型，并保留默认支持的类型。"
                        }.into());
                    }
                    Err(error) => {
                        this.association_error = Some(format!("配置失败：{error:#}。可重试注册并打开设置。"));
                    }
                }
                cx.notify();
            });
        }).detach();
    }
}
