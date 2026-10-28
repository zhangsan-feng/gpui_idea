mod appearance;
mod core;
mod language_servers;
mod system;
mod ui;

use crate::domain::settings::GeneralSettings;
use gpui_kit::component::input::InputState;
use gpui_kit::{AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};

pub(in crate::gui) struct SettingsWindow {
    page: SettingsPage,
    fonts: Vec<String>,
    system_status: Option<crate::domain::settings::ContextMenuStatus>,
    system_busy: bool,
    system_message: Option<String>,
    system_error: Option<String>,
    servers: Vec<crate::domain::language_server::ManagedServer>,
    server_busy: bool,
    server_cleaning: bool,
    server_actions: std::collections::HashMap<String, language_servers::ServerOperationState>,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    Appearance,
    General,
    LanguageServers,
    System,
}

impl SettingsWindow {
    pub(in crate::gui) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = crate::data_context::DATA_CONTEXT.settings.get();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("搜索设置"));
        let subscription = cx.observe(&search, |this, search, cx| {
            let query = search.read(cx).value().trim().to_lowercase();
            if !query.is_empty() {
                if "外观 字体 字号 主题 浅色 深色 font theme appearance".contains(&query)
                {
                    this.page = SettingsPage::Appearance;
                } else if "常规配置 自动格式化 保存 代理 proxy 网络 下载".contains(&query)
                {
                    this.page = SettingsPage::General;
                } else if "系统设置 windows 右键 菜单 文件管理器 集成 explorer".contains(&query)
                {
                    this.page = SettingsPage::System;
                    if this.system_status.is_none() && !this.system_busy {
                        this.manage_context_menu(
                            crate::domain::settings::ContextMenuAction::Refresh,
                            cx,
                        );
                    }
                } else {
                    this.page = SettingsPage::LanguageServers;
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
            fonts: cx.text_system().all_font_names(),
            page: SettingsPage::General,
            system_status: crate::data_context::DATA_CONTEXT.settings.context_menu(),
            system_busy: false,
            system_message: None,
            system_error: None,
            servers: crate::application::APPLICATION.settings.managed_servers(),
            server_busy: false,
            server_cleaning: false,
            server_actions: std::collections::HashMap::new(),
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

impl Render for SettingsWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        use gpui_kit::{ParentElement as _, Styled as _};
        gpui_kit::component::v_flex()
            .size_full()
            .font_family(crate::component::appearance::ui_font())
            .child(crate::component::operation_title_bar::render(
                "设置",
                !self.saving && !self.system_busy,
                cx,
            ))
            .child(self.render_view(cx))
    }
}
