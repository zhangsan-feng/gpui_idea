use super::super::*;
use crate::domain::language_server::ServerManagementAction;

impl WorkspaceSettings {
    pub(in crate::gui::workspace::settings) fn manage_servers(
        &mut self,
        action: ServerManagementAction,
        cx: &mut Context<Self>,
    ) {
        if self.server_busy {
            return;
        }
        let message = match &action {
            ServerManagementAction::Refresh => "服务器列表已刷新",
            ServerManagementAction::CleanHistory => {
                "历史 LSP 版本已清理，每个服务器仅保留最新的已安装版本。"
            }
            ServerManagementAction::Update(_) => {
                "已安装最新兼容版本；固定版本选择保持不变，重新打开项目后生效。"
            }
            ServerManagementAction::Select { .. } => "版本选择已保存，重新打开项目后生效。",
        }
        .to_owned();
        self.server_busy = true;
        self.server_error = None;
        self.server_message = None;
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
                this.server_busy = false;
                match result {
                    Ok(servers) => {
                        this.servers = servers;
                        this.server_message = Some(message);
                    }
                    Err(error) => this.server_error = Some(format!("操作失败：{error:#}")),
                }
                cx.notify();
            });
        })
        .detach();
    }
}
