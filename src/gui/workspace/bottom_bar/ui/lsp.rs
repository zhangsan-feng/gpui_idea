use super::*;
use crate::domain::language_server::LanguageServerPhase;

pub(super) fn installing() -> bool {
    DATA_CONTEXT
        .language_servers()
        .iter()
        .any(|server| server.phase == LanguageServerPhase::Installing)
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
        .w(px(380.))
        .gap_2()
        .text_size(px(12.))
        .text_color(color(TEXT))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().font_weight(FontWeight::SEMIBOLD).child("语言服务"))
                .child(div().text_size(px(11.)).text_color(color(MUTED)).child(
                    if installing > 0 {
                        format!("安装中 {installing} 个 · 已启动 {running} 个")
                    } else {
                        format!("已启动 {running} 个")
                    },
                )),
        )
        .child(
            div()
                .text_size(px(11.))
                .text_color(color(MUTED))
                .child(detail.to_owned()),
        )
        .child(div().h(px(1.)).bg(color(BORDER)))
        .when(servers.is_empty(), |panel| {
            panel.child(
                div()
                    .py_2()
                    .text_color(color(MUTED))
                    .child("当前没有已启动的语言服务"),
            )
        })
        .child(
            v_flex()
                .id("language-server-details-list")
                .max_h(px(320.))
                .overflow_y_scroll()
                .gap_3()
                .children(servers.into_iter().map(|server| {
                    let (phase, phase_color) = match server.phase {
                        LanguageServerPhase::Installing => ("正在安装", ACCENT),
                        LanguageServerPhase::Starting => ("初始化中", ACCENT),
                        LanguageServerPhase::Ready => ("已启动", 0x16a34a),
                        LanguageServerPhase::Failed => ("安装/启动失败", 0xdc2626),
                    };
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .items_center()
                                .gap_2()
                                .child(div().font_weight(FontWeight::MEDIUM).child(server.language))
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(color(phase_color))
                                        .child(phase),
                                ),
                        )
                        .child(div().child(server.name))
                        .when(!server.initialized, |row| {
                            row.child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(color(phase_color))
                                    .child(server.detail),
                            )
                        })
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(color(MUTED))
                                .child(format!(
                                    "版本：{}",
                                    server.version.as_deref().unwrap_or("版本未提供")
                                )),
                        )
                        .when_some(server.install_directory, |row, path| {
                            row.child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(color(MUTED))
                                    .child(format!("安装目录：{}", path.display())),
                            )
                        })
                        .when_some(server.executable, |row, path| {
                            row.child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(color(MUTED))
                                    .child(format!("启动程序：{}", path.display())),
                            )
                        })
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(color(MUTED))
                                .truncate()
                                .child(format!("工作区：{}", server.root.display())),
                        )
                })),
        )
}
