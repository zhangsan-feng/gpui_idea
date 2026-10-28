use super::*;
use crate::domain::language_server::{LanguageServerInfo, LanguageServerPhase};

pub(super) fn installation_detail() -> Option<String> {
    let details = DATA_CONTEXT
        .language_servers()
        .iter()
        .filter(|server| server.phase == LanguageServerPhase::Installing)
        .map(|server| format!("{} · {}", server.name, server.detail))
        .collect::<Vec<_>>();
    (!details.is_empty()).then(|| details.join("；"))
}

pub(super) fn details(detail: &str) -> Div {
    let mut servers = DATA_CONTEXT.language_servers();
    servers.sort_by(|left, right| {
        left.language
            .cmp(&right.language)
            .then(left.root.cmp(&right.root))
    });
    let running = servers.iter().filter(|server| server.initialized).count();
    let installing = servers
        .iter()
        .filter(|server| server.phase == LanguageServerPhase::Installing)
        .count();
    v_flex()
        .w(px(440.))
        .rounded_lg()
        .border_1()
        .border_color(crate::component::appearance::border(BORDER))
        .bg(crate::component::appearance::background(EDITOR_BG))
        .shadow_lg()
        .text_size(px(12.))
        .line_height(px(18.))
        .text_color(crate::component::appearance::foreground(TEXT))
        .child(
            v_flex()
                .flex_shrink_0()
                .p_3()
                .gap_2()
                .border_b_1()
                .border_color(crate::component::appearance::border(BORDER))
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("语言服务"),
                        )
                        .child(div().text_size(px(11.)).text_color(crate::component::appearance::foreground(MUTED)).child(
                            if installing > 0 {
                                format!("安装中 {installing} · 已启动 {running}")
                            } else {
                                format!("已启动 {running} 个")
                            },
                        )),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(crate::component::appearance::foreground(MUTED))
                        .child(detail.to_owned()),
                ),
        )
        .when(servers.is_empty(), |panel| {
            panel.child(
                div()
                    .p_4()
                    .text_color(crate::component::appearance::foreground(MUTED))
                    .child("当前没有已启动的语言服务"),
            )
        })
        .when(!servers.is_empty(), |panel| {
            panel.child(
                v_flex()
                    .id("language-server-details-list")
                    .max_h(px(340.))
                    .overflow_y_scroll()
                    .p_3()
                    .gap_2()
                    .children(servers.into_iter().map(server_card)),
            )
        })
}

fn server_card(server: LanguageServerInfo) -> Div {
    let detail = match &server.runtime_status {
        Some(LanguageServerStatus::Warning(detail) | LanguageServerStatus::Error(detail)) => {
            Some(detail.clone())
        }
        _ if !server.initialized && !server.detail.is_empty() => Some(server.detail.clone()),
        _ => None,
    };
    let (phase, phase_color, phase_bg) = match (&server.phase, &server.runtime_status) {
        (_, Some(LanguageServerStatus::Indexing)) => ("索引中", ACCENT, 0xeff6ff),
        (_, Some(LanguageServerStatus::Warming)) => ("预热中", ACCENT, 0xeff6ff),
        (_, Some(LanguageServerStatus::Ready)) => ("就绪", 0x15803d, 0xf0fdf4),
        (_, Some(LanguageServerStatus::Warning(_))) => ("警告", 0xd97706, 0xfffbeb),
        (_, Some(LanguageServerStatus::Error(_))) => ("异常", 0xdc2626, 0xfef2f2),
        (phase, _) => match phase {
            LanguageServerPhase::Installing => ("正在安装", ACCENT, 0xeff6ff),
            LanguageServerPhase::Starting => ("初始化中", ACCENT, 0xeff6ff),
            LanguageServerPhase::Ready => ("已启动", 0x15803d, 0xf0fdf4),
            LanguageServerPhase::Failed => ("安装/启动失败", 0xdc2626, 0xfef2f2),
        },
    };
    v_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .p_3()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(crate::component::appearance::border(BORDER))
        .child(
            h_flex()
                .items_start()
                .justify_between()
                .gap_2()
                .child(
                    v_flex()
                        .min_w_0()
                        .flex_1()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(server.language),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(crate::component::appearance::foreground(MUTED))
                                .child(server.name),
                        ),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .px_2()
                        .rounded_sm()
                        .bg(crate::component::appearance::background(phase_bg))
                        .text_size(px(10.))
                        .text_color(crate::component::appearance::foreground(phase_color))
                        .child(phase),
                ),
        )
        .when_some(detail, |card, detail| {
            card.child(
                div()
                    .text_size(px(11.))
                    .text_color(crate::component::appearance::foreground(phase_color))
                    .child(detail),
            )
        })
        .child(
            v_flex()
                .gap_1p5()
                .child(info_row(
                    "版本",
                    server.version.unwrap_or_else(|| "版本未提供".into()),
                ))
                .when_some(server.executable, |rows, path| {
                    rows.child(info_row("启动程序", path.display().to_string()))
                }),
        )
}

fn info_row(label: &'static str, value: String) -> Div {
    h_flex()
        .w_full()
        .items_start()
        .gap_2()
        .text_size(px(11.))
        .line_height(px(17.))
        .child(
            div()
                .w(px(52.))
                .flex_shrink_0()
                .text_color(crate::component::appearance::foreground(MUTED))
                .child(label),
        )
        .child(div().min_w_0().flex_1().whitespace_normal().child(value))
}
