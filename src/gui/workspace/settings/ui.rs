use super::*;
use gpui_kit::{
    assets::IconName,
    component::{
        Disableable as _, Icon, Sizable as _, Size,
        button::{Button, ButtonVariants as _},
        checkbox::Checkbox,
        h_flex,
        input::Input,
        v_flex,
    },
    prelude::*,
    *,
};

const BACKGROUND: u32 = 0xf7f8fa;
const BORDER: u32 = 0xe5e7eb;
const TEXT: u32 = 0x20242b;
const MUTED: u32 = 0x6b7280;

fn settings_icon(name: IconName, size: f32) -> Icon {
    Icon::new(name).with_size(Size::Size(px(size)))
}

impl WorkspaceSettings {
    pub(super) fn render_view(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .flex_1()
            .min_h_0()
            .bg(rgb(BACKGROUND))
            .text_size(px(13.))
            .text_color(rgb(TEXT))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_start()
                    .child(self.sidebar(cx))
                    .child(match self.page {
                        SettingsPage::LanguageServers => self.render_servers(cx),
                        SettingsPage::System => self.render_system(cx),
                        SettingsPage::General => self.content(cx),
                    }),
            )
            .when_some(self.error.clone(), |content, error| {
                content.child(div().px_5().py_2().text_color(rgb(0xd44352)).child(error))
            })
            .child(self.footer(cx))
    }

    fn matches_search(&self, cx: &Context<Self>) -> bool {
        let query = self.search.read(cx).value().trim().to_lowercase();
        query.is_empty() || "常规配置 自动格式化 保存 代理 proxy 网络 下载".contains(&query)
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> Div {
        let matches = self.matches_search(cx);
        let query = self.search.read(cx).value().trim().to_lowercase();
        let matches_system = query.is_empty()
            || "系统设置 windows 右键 菜单 文件管理器 集成 explorer".contains(&query);
        let matches_lsp = query.is_empty() || "lsp 管理 语言服务器 server 版本 安装 更新 rust go java toml markdown python javascript typescript html css json yaml php intelephense pyright taplo marksman".contains(&query)
            || self.servers.iter().any(|server| format!("{} {}", server.name, server.languages).to_lowercase().contains(&query));
        v_flex()
            .w(px(220.))
            .h_full()
            .flex_shrink_0()
            .min_h_0()
            .bg(rgb(0xf8fafc))
            .border_r_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .p_3()
                    .child(Input::new(&self.search).prefix(settings_icon(IconName::Search, 14.))),
            )
            .child(
                v_flex()
                    .id("settings-category-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_2()
                    .pb_3()
                    .gap_1()
                    .when(matches, |list| {
                        list.child(
                            h_flex()
                                .id("general-settings-category")
                                .h(px(36.))
                                .items_center()
                                .gap_2()
                                .px_3()
                                .rounded_md()
                                .text_color(rgb(0x4b5563))
                                .when(self.page == SettingsPage::General, |row| {
                                    row.bg(rgb(0xe8efff))
                                        .text_color(rgb(0x1d4ed8))
                                        .font_weight(FontWeight::SEMIBOLD)
                                })
                                .cursor_pointer()
                                .hover(|row| row.bg(rgb(0xeef2f7)))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.page = SettingsPage::General;
                                    cx.notify();
                                }))
                                .child(settings_icon(IconName::SlidersHorizontal, 15.))
                                .child("常规配置"),
                        )
                    })
                    .when(matches_lsp, |list| {
                        list.child(
                            h_flex()
                                .id("lsp-settings-category")
                                .h(px(36.))
                                .items_center()
                                .px_3()
                                .font_weight(FontWeight::SEMIBOLD)
                                .rounded_md()
                                .text_color(rgb(0x4b5563))
                                .gap_2()
                                .when(self.page == SettingsPage::LanguageServers, |row| {
                                    row.bg(rgb(0xe8efff)).text_color(rgb(0x1d4ed8))
                                })
                                .cursor_pointer()
                                .hover(|row| row.bg(rgb(0xeef2f7)))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.page = SettingsPage::LanguageServers;
                                    cx.notify();
                                }))
                                .child(settings_icon(IconName::Code, 15.))
                                .child("LSP 管理"),
                        )
                    })
                    .when(matches_system, |list| {
                        list.child(
                            h_flex()
                                .id("system-settings-category")
                                .h(px(36.))
                                .items_center()
                                .gap_2()
                                .px_3()
                                .rounded_md()
                                .text_color(rgb(0x4b5563))
                                .when(self.page == SettingsPage::System, |row| {
                                    row.bg(rgb(0xe8efff))
                                        .text_color(rgb(0x1d4ed8))
                                        .font_weight(FontWeight::SEMIBOLD)
                                })
                                .cursor_pointer()
                                .hover(|row| row.bg(rgb(0xeef2f7)))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.page = SettingsPage::System;
                                    this.manage_context_menu(
                                        crate::domain::settings::ContextMenuAction::Refresh,
                                        cx,
                                    );
                                    cx.notify();
                                }))
                                .child(settings_icon(IconName::Settings, 15.))
                                .child("系统设置"),
                        )
                    })
                    .when(!matches && !matches_lsp && !matches_system, |list| {
                        list.child(
                            div()
                                .px_4()
                                .py_3()
                                .text_color(rgb(MUTED))
                                .child("未找到匹配的设置"),
                        )
                    }),
            )
    }

    fn content(&self, cx: &mut Context<Self>) -> Div {
        v_flex().h_full().flex_1().min_w_0().min_h_0()
            .child(h_flex().h(px(60.)).flex_shrink_0().px_5().gap_2().text_size(px(15.)).items_center().font_weight(FontWeight::SEMIBOLD)
                .child("常规配置"))
            .child(v_flex().flex_1().min_h_0().mx_4().mb_4().pt_5().gap_3().border_t_1().border_color(rgb(BORDER))
                .id("general-settings-content").overflow_y_scroll()
                .when(self.matches_search(cx), |content| content
                    .child(Checkbox::new("auto-format-on-save").checked(self.draft.auto_format).disabled(self.saving)
                        .label("保存时自动格式化")
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.draft.auto_format = *checked; this.error = None; cx.notify();
                        })))
                    .child(div().pl_6().text_size(px(12.)).text_color(rgb(MUTED))
                        .child("保存文件时使用当前语言的 LSP 格式化，适用于手动保存和自动保存。"))
                    .child(div().pl_6().text_size(px(12.)).text_color(rgb(MUTED))
                        .child("语言服务不支持格式化或格式化失败时，仍会保存原内容。"))
                    .child(self.proxy_settings(cx))))
    }

    fn proxy_settings(&self, cx: &mut Context<Self>) -> Div {
        v_flex()
            .mt_4()
            .p_4()
            .gap_3()
            .rounded_lg()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(BORDER))
            .child(div().font_weight(FontWeight::SEMIBOLD).child("下载代理"))
            .child(
                Checkbox::new("lsp-download-proxy")
                    .checked(self.draft.lsp_proxy.enabled)
                    .disabled(self.saving || self.system_busy)
                    .label("为 LSP 下载使用代理")
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.draft.lsp_proxy.enabled = *checked;
                        this.error = None;
                        cx.notify();
                    })),
            )
            .child(
                v_flex()
                    .gap_2()
                    .pl_6()
                    .child(div().text_color(rgb(MUTED)).child("代理地址"))
                    .child(
                        Input::new(&self.proxy_url)
                            .disabled(self.saving || !self.draft.lsp_proxy.enabled),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(MUTED))
                            .child("支持 HTTP / HTTPS 代理，包含协议、主机和端口。"),
                    ),
            )
            .child(
                div()
                    .pl_6()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child("范围：LSP 版本检查、安装与更新下载，包含 npm 和 Go 安装。"),
            )
            .child(
                div()
                    .pl_6()
                    .text_size(px(12.))
                    .text_color(rgb(MUTED))
                    .child("应用后对新请求生效；关闭后沿用系统环境代理设置。"),
            )
    }

    fn footer(&self, cx: &mut Context<Self>) -> Div {
        h_flex()
            .h(px(52.))
            .flex_shrink_0()
            .px_5()
            .bg(rgb(0xffffff))
            .gap_2()
            .items_center()
            .border_t_1()
            .border_color(rgb(BORDER))
            .child(div().flex_1())
            .when(self.saving, |row| {
                row.child(div().text_color(rgb(MUTED)).child("正在保存…"))
            })
            .child(
                Button::new("settings-ok")
                    .primary()
                    .w(px(76.))
                    .label("确定")
                    .disabled(self.saving || self.system_busy)
                    .on_click(cx.listener(|this, _, window, cx| this.apply(true, window, cx))),
            )
            .child(
                Button::new("settings-cancel")
                    .w(px(76.))
                    .label("取消")
                    .disabled(self.saving || self.system_busy)
                    .on_click(|_, window, _| window.remove_window()),
            )
            .child(
                Button::new("settings-apply")
                    .w(px(76.))
                    .label("应用")
                    .disabled(self.saving || self.system_busy || self.draft == self.saved)
                    .on_click(cx.listener(|this, _, window, cx| this.apply(false, window, cx))),
            )
    }
}
