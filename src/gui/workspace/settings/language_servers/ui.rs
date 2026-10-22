use super::super::*;
use super::ServerOperationState;
use crate::domain::language_server::ServerManagementAction;
use gpui_kit::{
    component::{
        Disableable as _, Sizable as _,
        button::{Button, ButtonVariants as _},
        h_flex, v_flex,
    },
    prelude::*,
    *,
};

impl WorkspaceSettings {
    pub(in crate::gui::workspace::settings) fn render_servers(
        &self,
        cx: &mut Context<Self>,
    ) -> Div {
        let query = self.search.read(cx).value().trim().to_lowercase();
        let all = query.is_empty() || "lsp 管理 语言服务器 server 版本 安装 更新".contains(&query);
        let no_matches = !self.server_busy
            && !self.servers.iter().any(|server| {
                all || format!("{} {}", server.name, server.languages)
                    .to_lowercase()
                    .contains(&query)
            });
        v_flex()
            .h_full()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(
                h_flex()
                    .h(px(60.))
                    .px_5()
                    .gap_3()
                    .items_center()
                    .flex_shrink_0()
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("LSP 管理"),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("refresh-managed-servers")
                            .ghost()
                            .small()
                            .icon(gpui_kit::assets::IconName::RefreshCw)
                            .label("刷新列表")
                            .disabled(self.server_busy)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.manage_servers(ServerManagementAction::Refresh, cx)
                            })),
                    )
                    .child(
                        Button::new("clean-managed-server-history")
                            .ghost()
                            .small()
                            .icon(gpui_kit::assets::IconName::Trash)
                            .label("清理历史 LSP 版本")
                            .disabled(self.server_busy || self.any_server_busy())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.manage_servers(ServerManagementAction::CleanHistory, cx)
                            })),
                    ),
            )
            .child(
                v_flex()
                    .id("managed-server-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_5()
                    .pb_5()
                    .gap_3()
                    .when(self.server_busy, |list| {
                        list.child(div().text_color(rgb(0x2563eb)).child("正在处理，请稍候…"))
                    })
                    .when_some(self.server_message.clone(), |list, text| {
                        list.child(div().text_color(rgb(0x15803d)).child(text))
                    })
                    .when_some(self.server_error.clone(), |list, text| {
                        list.child(div().text_color(rgb(0xdc2626)).child(text))
                    })
                    .when(no_matches, |list| {
                        list.child(
                            div()
                                .text_color(rgb(0x6b7280))
                                .child("未找到匹配的语言服务器"),
                        )
                    })
                    .children(
                        self.servers
                            .iter()
                            .filter(|server| {
                                all || format!("{} {}", server.name, server.languages)
                                    .to_lowercase()
                                    .contains(&query)
                            })
                            .map(|server| self.server_card(server, cx)),
                    ),
            )
    }

    fn server_card(
        &self,
        server: &crate::domain::language_server::ManagedServer,
        cx: &mut Context<Self>,
    ) -> Div {
        let key = server.key.clone();
        let auto_key = key.clone();
        let busy = self.server_is_busy(&key);
        let disabled = busy || self.server_cleaning;
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(rgb(0xe5e7eb))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(server.name.clone()),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(12.))
                                    .text_color(rgb(0x6b7280))
                                    .child(server.languages.clone()),
                            ),
                    )
                    .child(
                        Button::new(SharedString::from(format!("update-{key}")))
                            .small()
                            .label(if busy {
                                "处理中…"
                            } else if server.versions.is_empty() {
                                "安装"
                            } else {
                                "检查并更新"
                            })
                            .disabled(disabled)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.manage_servers(ServerManagementAction::Update(key.clone()), cx)
                            })),
                    ),
            )
            .when(
                server.pinned.is_some() || server.configuration_error.is_some(),
                |card| {
                    card.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(12.))
                                    .text_color(rgb(0x64748b))
                                    .child(if server.configuration_error.is_some() {
                                        "版本配置异常"
                                    } else {
                                        "固定版本 · 启动时不自动更新"
                                    }),
                            )
                            .when(
                                server.pinned.is_some() || server.configuration_error.is_some(),
                                |row| {
                                    row.child(
                                        Button::new(SharedString::from(format!("auto-{auto_key}")))
                                            .small()
                                            .label("恢复自动更新")
                                            .disabled(disabled)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.manage_servers(
                                                    ServerManagementAction::Select {
                                                        key: auto_key.clone(),
                                                        directory: None,
                                                    },
                                                    cx,
                                                )
                                            })),
                                    )
                                },
                            ),
                    )
                },
            )
            .when_some(self.server_actions.get(&server.key), |card, state| {
                let (text, color) = match state {
                    ServerOperationState::Busy(text) => ((*text).to_owned(), 0x2563eb),
                    ServerOperationState::Success(text) => (text.clone(), 0x15803d),
                    ServerOperationState::Error(text) => (text.clone(), 0xdc2626),
                };
                card.child(div().text_size(px(12.)).text_color(rgb(color)).child(text))
            })
            .when_some(server.configuration_error.clone(), |card, error| {
                card.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0xdc2626))
                        .child(error),
                )
            })
            .when(server.versions.is_empty(), |card| {
                card.child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(0x6b7280))
                        .child("尚未安装；将根据本机运行环境选择兼容版本。"),
                )
            })
            .when(
                server
                    .pinned
                    .as_ref()
                    .is_some_and(|pin| !server.versions.iter().any(|v| &v.directory == pin)),
                |card| {
                    card.child(
                        div()
                            .text_color(rgb(0xdc2626))
                            .child("固定版本已丢失，请重新选择或恢复自动更新。"),
                    )
                },
            )
            .children(server.versions.iter().map(|version| {
                let selected = server.pinned.as_ref() == Some(&version.directory);
                let key = server.key.clone();
                let directory = version.directory.clone();
                h_flex()
                    .gap_2()
                    .items_center()
                    .min_h(px(34.))
                    .pt_2()
                    .border_t_1()
                    .border_color(rgb(0xf3f4f6))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .font_family("Cascadia Mono")
                            .child(version.version.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(if selected { 0xeff6ff } else { 0xf1f5f9 }))
                            .text_color(rgb(if selected { 0x2563eb } else { 0x64748b }))
                            .child(if selected { "已固定" } else { "已安装" }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("select-{directory}")))
                            .small()
                            .label(if selected {
                                "当前固定版本"
                            } else {
                                "使用此版本"
                            })
                            .disabled(disabled || selected)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.manage_servers(
                                    ServerManagementAction::Select {
                                        key: key.clone(),
                                        directory: Some(directory.clone()),
                                    },
                                    cx,
                                )
                            })),
                    )
            }))
    }
}
