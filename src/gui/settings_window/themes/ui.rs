use super::super::SettingsWindow;
use gpui_kit::{
    component::{
        Disableable as _, Selectable as _,
        button::Button,
        h_flex,
        input::Input,
        menu::{DropdownMenu, PopupMenuItem},
        v_flex,
    },
    prelude::*,
    *,
};

impl SettingsWindow {
    pub(in crate::gui::settings_window) fn render_themes(&self, cx: &mut Context<Self>) -> Div {
        let busy = self.saving || self.system_busy || self.themes.busy;
        let appearance = &self.draft.appearance;
        let selected = appearance.selected_theme.as_ref().and_then(|id| {
            appearance
                .imported_themes
                .iter()
                .find(|theme| &theme.id == id)
        });
        let current = selected
            .map(|theme| theme.name.as_str())
            .unwrap_or("默认主题");
        let mut content = v_flex()
            .gap_3()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("VS Code 主题"),
            )
            .child(div().child(format!("当前选择：{current}")))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("themes-installed")
                            .label("已安装")
                            .selected(!self.themes.online)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.themes.online = false;
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("themes-online")
                            .label("在线主题")
                            .selected(self.themes.online)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.themes.online = true;
                                if !this.themes.loaded {
                                    this.search_themes(0, cx);
                                }
                                cx.notify();
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("theme-import")
                            .label("导入 JSON / VSIX")
                            .disabled(busy)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.import_theme(window, cx)),
                            ),
                    ),
            )
            .when_some(self.themes.message.clone(), |view, message| {
                view.child(div().child(message))
            })
            .when(self.themes.busy, |view| {
                view.child(div().child("正在处理主题，请稍候…"))
            });
        if self.themes.online {
            content = content
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(Input::new(&self.themes.query).disabled(busy)),
                        )
                        .child(
                            Button::new("themes-search")
                                .label("搜索")
                                .disabled(busy)
                                .on_click(cx.listener(|this, _, _, cx| this.search_themes(0, cx))),
                        ),
                )
                .child(
                    div()
                        .text_size(crate::gui::component::appearance::ui_text_size(12.))
                        .text_color(crate::component::appearance::foreground(0x6b7280))
                        .child("来源：Open VSX。仅显示颜色主题包；下载后在“已安装”中选择配色。"),
                );
            for item in &self.themes.results {
                let pending = item.clone();
                let installed = appearance
                    .imported_themes
                    .iter()
                    .any(|theme| theme.source == item.id);
                content = content.child(
                    crate::component::panel::card(0xe5e7eb)
                        .child(
                            h_flex()
                                .gap_3()
                                .items_center()
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .gap_1()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child(item.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(
                                                    crate::gui::component::appearance::ui_text_size(
                                                        12.,
                                                    ),
                                                )
                                                .child(format!(
                                                    "{} · v{} · 下载 {}",
                                                    item.id, item.version, item.downloads
                                                )),
                                        ),
                                )
                                .child(
                                    Button::new(SharedString::from(format!(
                                        "theme-download-{}",
                                        item.id
                                    )))
                                    .label(if installed { "重新下载" } else { "下载" })
                                    .disabled(busy)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.download_theme(pending.clone(), cx)
                                    })),
                                ),
                        )
                        .child(
                            div()
                                .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                .child(item.description.clone()),
                        ),
                );
            }
            content = content
                .when(
                    self.themes.loaded && self.themes.results.is_empty(),
                    |view| {
                        view.child(
                            div().child("当前没有可显示的颜色主题，请尝试其他关键词或下一页。"),
                        )
                    },
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("themes-previous")
                                .label("上一页")
                                .disabled(busy || self.themes.offset == 0)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.previous_theme_page(cx)),
                                ),
                        )
                        .child(div().child(format!("第 {} 页", self.themes.page_offsets.len() + 1)))
                        .child(
                            Button::new("themes-next")
                                .label("下一页")
                                .disabled(busy || self.themes.next_offset.is_none())
                                .on_click(cx.listener(|this, _, _, cx| this.next_theme_page(cx))),
                        ),
                );
        } else {
            let default_selected = appearance.selected_theme.is_none();
            content = content.child(
                crate::component::panel::card(0xe5e7eb).child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(div().font_weight(FontWeight::SEMIBOLD).child("默认主题"))
                                .child(
                                    div()
                                        .text_size(crate::gui::component::appearance::ui_text_size(
                                            12.,
                                        ))
                                        .child("应用内置 · 始终可用"),
                                ),
                        )
                        .child(
                            Button::new("theme-select-default")
                                .label(if default_selected {
                                    "已选择"
                                } else {
                                    "选择"
                                })
                                .selected(default_selected)
                                .disabled(busy)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.select_default_theme(cx)),
                                ),
                        ),
                ),
            );
            if appearance.imported_themes.is_empty() {
                content = content
                    .child(div().child("可从在线列表下载更多配色，或导入本地 JSON / VSIX 文件。"));
            }
            for package in crate::application::APPLICATION
                .settings
                .installed_theme_packages(&self.draft)
            {
                content = content.child(self.installed_theme_card(package, busy, cx));
            }
        }
        content.child(
            div()
                .text_size(crate::gui::component::appearance::ui_text_size(12.))
                .text_color(crate::component::appearance::foreground(0x6b7280))
                .child("选择、导入或卸载主题后，点击“应用”或“确定”生效并保存；点击“取消”放弃未保存的主题更改。"),
        )
    }
    fn installed_theme_card(
        &self,
        package: crate::domain::theme::InstalledThemePackage<'_>,
        busy: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let selected = package.themes.iter().copied().find(|theme| {
            self.draft.appearance.selected_theme.as_deref() == Some(theme.id.as_str())
        });
        let label = selected
            .map(|theme| theme.name.clone())
            .unwrap_or_else(|| "选择配色".into());
        let options: Vec<_> = package
            .themes
            .iter()
            .map(|theme| {
                (
                    theme.id.clone(),
                    theme.name.clone(),
                    selected.is_some_and(|selected| selected.id == theme.id),
                )
            })
            .collect();
        let notes = selected
            .or_else(|| package.themes.first().copied())
            .map(|theme| theme.notes.clone())
            .unwrap_or_default();
        let source = package.source.to_owned();
        let view = cx.entity().downgrade();
        crate::component::panel::card(0xe5e7eb)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(package.name.to_owned()),
                            )
                            .child(
                                div()
                                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                    .child(format!(
                                        "{} · {} 套配色",
                                        package.source,
                                        package.themes.len()
                                    )),
                            ),
                    )
                    .child(
                        Button::new(SharedString::from(format!("theme-variants-{source}")))
                            .label(label)
                            .selected(selected.is_some())
                            .disabled(busy)
                            .w(px(260.))
                            .dropdown_menu(move |mut menu, _, _| {
                                menu = menu.min_w(px(260.)).max_h(px(320.)).scrollable(true);
                                for (id, name, selected) in &options {
                                    let id = id.clone();
                                    let view = view.clone();
                                    menu = menu.item(
                                        PopupMenuItem::new(name.clone())
                                            .checked(*selected)
                                            .on_click(move |_, _, cx| {
                                                let _ = view.update(cx, |this, cx| {
                                                    this.select_theme(&id, cx)
                                                });
                                            }),
                                    );
                                }
                                menu
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("theme-uninstall-{source}")))
                            .label("卸载")
                            .disabled(busy)
                            .on_click(
                                cx.listener(move |this, _, _, cx| {
                                    this.uninstall_theme(&source, cx)
                                }),
                            ),
                    ),
            )
            .children(notes.into_iter().map(|note| {
                div()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(0x6b7280))
                    .child(note)
            }))
    }
}
