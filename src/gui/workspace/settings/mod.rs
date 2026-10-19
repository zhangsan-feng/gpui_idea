mod core;
mod language_servers;
mod ui;

use crate::domain::settings::GeneralSettings;
use gpui_kit::component::input::InputState;
use gpui_kit::{AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};

pub(super) struct WorkspaceSettings {
    lsp_page: bool,
    servers: Vec<crate::domain::language_server::ManagedServer>,
    server_busy: bool,
    server_message: Option<String>,
    server_error: Option<String>,
    draft: GeneralSettings,
    saved: GeneralSettings,
    saving: bool,
    error: Option<String>,
    search: Entity<InputState>,
    proxy_url: Entity<InputState>,
    _proxy_subscription: Subscription,
    _search_subscription: Subscription,
}

impl WorkspaceSettings {
    pub(super) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = crate::data_context::DATA_CONTEXT.settings.get();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("搜索设置"));
        let subscription = cx.observe(&search, |this, search, cx| {
            let query = search.read(cx).value().trim().to_lowercase();
            if !query.is_empty() {
                if "常规配置 自动格式化 保存 代理 proxy 网络 下载".contains(&query)
                {
                    this.lsp_page = false;
                } else {
                    this.lsp_page = true;
                }
            }
            cx.notify();
        });
        let proxy_url = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("http://127.0.0.1:7890")
                .default_value(settings.lsp_proxy.url.clone())
        });
        let proxy_subscription = cx.subscribe(&proxy_url, |this, input, event, cx| {
            if matches!(event, gpui_kit::component::input::InputEvent::Change) {
                this.draft.lsp_proxy.url = input.read(cx).value().to_string();
                this.error = None;
                cx.notify();
            }
        });
        cx.spawn(async move |this, cx| {
            let _ = this.update(cx, |this, cx| {
                this.manage_servers(
                    crate::domain::language_server::ServerManagementAction::Refresh,
                    cx,
                )
            });
        })
        .detach();
        Self {
            lsp_page: false,
            servers: crate::application::APPLICATION.settings.managed_servers(),
            server_busy: false,
            server_message: None,
            server_error: None,
            draft: settings.clone(),
            saved: settings,
            saving: false,
            error: None,
            search,
            proxy_url,
            _proxy_subscription: proxy_subscription,
            _search_subscription: subscription,
        }
    }
}

impl Render for WorkspaceSettings {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::{ParentElement as _, Styled as _};
        gpui_kit::component::v_flex()
            .size_full()
            .font_family("Microsoft YaHei UI")
            .child(crate::component::operation_title_bar::render(
                "设置",
                !self.saving,
                cx,
            ))
            .child(self.render_view(cx))
    }
}
