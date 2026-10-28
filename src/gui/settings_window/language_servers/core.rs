use super::super::*;
use super::ServerOperationState;
use crate::domain::language_server::ServerManagementAction;

impl SettingsWindow {
    pub(in crate::gui) fn manage_servers(
        &mut self,
        action: ServerManagementAction,
        cx: &mut Context<Self>,
    ) {
        let key = match &action {
            ServerManagementAction::Update(key) | ServerManagementAction::Select { key, .. } => {
                Some(key.clone())
            }
            _ => None,
        };
        let cleaning = matches!(&action, ServerManagementAction::CleanHistory);
        if self.server_cleaning
            || key.as_ref().is_some_and(|key| self.server_is_busy(key))
            || (key.is_none() && self.server_busy)
            || (cleaning && self.any_server_busy())
        {
            return;
        }
        let message = match &action {
            ServerManagementAction::Refresh => "服务器列表已刷新",
            ServerManagementAction::CleanHistory => {
                "历史版本和 Go / npm 安装缓存已清理，每个服务器仅保留最新的已安装版本。"
            }
            ServerManagementAction::Update(_) => {
                "已安装最新兼容版本；固定版本选择保持不变，重新打开项目后生效。"
            }
            ServerManagementAction::Select { .. } => "版本选择已保存，重新打开项目后生效。",
        }
        .to_owned();
        if let Some(key) = &key {
            let status = match &action {
                ServerManagementAction::Update(_) => "正在检查并安装最新兼容版本…",
                _ => "正在保存版本选择…",
            };
            self.server_actions
                .insert(key.clone(), ServerOperationState::Busy(status));
        } else {
            self.server_busy = true;
            self.server_cleaning = cleaning;
            self.server_error = None;
            self.server_message = None;
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = crate::application::lifecycle::run_application(async move {
                crate::application::APPLICATION
                    .settings
                    .manage_servers(action)
                    .await
            })
            .await;
            let _ = this.update(cx, |this, cx| {
                // Another operation may already have published a newer snapshot.
                this.servers = crate::application::APPLICATION.settings.managed_servers();
                if let Some(key) = key {
                    let state = match result {
                        Ok(_) => ServerOperationState::Success(message),
                        Err(error) => ServerOperationState::Error(format!("操作失败：{error:#}")),
                    };
                    this.server_actions.insert(key, state);
                } else {
                    this.server_busy = false;
                    this.server_cleaning = false;
                    match result {
                        Ok(_) => this.server_message = Some(message),
                        Err(error) => this.server_error = Some(format!("操作失败：{error:#}")),
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(in crate::gui) fn server_is_busy(&self, key: &str) -> bool {
        matches!(
            self.server_actions.get(key),
            Some(ServerOperationState::Busy(_))
        )
    }

    pub(in crate::gui) fn any_server_busy(&self) -> bool {
        self.server_actions
            .values()
            .any(|state| matches!(state, ServerOperationState::Busy(_)))
    }
}
