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
            let _ = this.update(cx, |this, cx| {
                this.system_busy = false;
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
}
