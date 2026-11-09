mod appearance;
mod core;
mod editor;
mod language_servers;
mod system;
mod themes;
mod ui;

use crate::domain::{history::SettingsPage, settings::GeneralSettings};
use gpui_kit::component::input::InputState;
use gpui_kit::{AppContext as _, Context, Entity, IntoElement, Render, Subscription, Window};

pub(in crate::gui) struct SettingsWindow {
    page: SettingsPage,
    fonts: Vec<String>,
    themes: themes::ThemeBrowser,
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

const EDITOR_SEARCH_TERMS: &str = "编辑器配置 编辑器显示 文本换行 文本超出 自动换行 折行 soft wrap word wrap editor 编辑器字体 编辑器字号 字体 字号 font size 代码格式化 自动格式化代码 自动化格式化代码 保存时自动格式化代码 format formatting";

const GENERAL_SEARCH_TERMS: &str = "常规配置 代理 proxy 网络 下载";
const APPEARANCE_SEARCH_TERMS: &str =
    "外观 界面字体 主题 浅色 深色 theme appearance vscode vsix open vsx 在线 已安装 导入";

impl SettingsWindow {
    pub(in crate::gui) fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let snapshot = crate::data_context::DATA_CONTEXT.settings.snapshot();
        let settings = snapshot.general.as_ref().clone();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("搜索设置"));
        let subscription = cx.observe(&search, |this, search, cx| {
            let query = search.read(cx).value().trim().to_lowercase();
            if !query.is_empty() {
                if APPEARANCE_SEARCH_TERMS.contains(&query) {
                    this.select_page(SettingsPage::Appearance, cx);
                } else if GENERAL_SEARCH_TERMS.contains(&query) {
                    this.select_page(SettingsPage::General, cx);
                } else if EDITOR_SEARCH_TERMS.contains(&query) {
                    this.select_page(SettingsPage::Editor, cx);
                } else if "系统设置 windows 右键 菜单 文件管理器 集成 explorer".contains(&query)
                {
                    this.select_page(SettingsPage::System, cx);
                    if this.system_status.is_none() && !this.system_busy {
                        this.manage_context_menu(
                            crate::domain::settings::ContextMenuAction::Refresh,
                            cx,
                        );
                    }
                } else {
                    this.select_page(SettingsPage::LanguageServers, cx);
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
        let theme_browser = themes::ThemeBrowser::new(window, cx);
        let mut this = Self {
            themes: theme_browser,
            fonts: cx.text_system().all_font_names(),
            page: crate::data_context::DATA_CONTEXT
                .history
                .snapshot()
                .settings_page,
            system_status: snapshot.context_menu.clone(),
            system_busy: false,
            system_message: None,
            system_error: None,
            servers: snapshot.servers.to_vec(),
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
        };
        tracing::debug!(page = ?this.page, "settings window page restored from history");
        this.start_subscribe(window, cx);
        if this.page == SettingsPage::System && this.system_status.is_none() && !this.system_busy {
            this.manage_context_menu(crate::domain::settings::ContextMenuAction::Refresh, cx);
        }
        this
    }

    fn start_subscribe(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut changes = crate::event_bus::EVENT_BUS
            .refresh_gui
            .subscribe(crate::event_bus::refresh_gui::GuiComponent::SettingsWindow);
        // Subscribe before rereading so initialization cannot miss a background publication.
        let snapshot = crate::data_context::DATA_CONTEXT.settings.snapshot();
        self.servers = snapshot.servers.to_vec();
        self.system_status = snapshot.context_menu.clone();
        cx.spawn_in(window, async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this
                    .update(cx, |this, cx| {
                        let snapshot = crate::data_context::DATA_CONTEXT.settings.snapshot();
                        this.servers = snapshot.servers.to_vec();
                        this.system_status = snapshot.context_menu.clone();
                        // Published background state must not overwrite the user's draft.
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
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
                !self.saving && !self.system_busy && !self.themes.busy(),
                cx,
            ))
            .child(self.render_view(cx))
    }
}
