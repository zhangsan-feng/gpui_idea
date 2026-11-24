use super::super::*;
use crate::domain::settings::ContextMenuAction;
use gpui_kit::{
    component::{
        Disableable as _,
        button::{Button, ButtonVariants as _},
        h_flex,
        input::Input,
        v_flex,
    },
    prelude::*,
    *,
};

impl SettingsWindow {
    pub(in crate::gui) fn render_system(&self, cx: &mut Context<Self>) -> Div {
        let supported = cfg!(windows)
            && self
                .system_status
                .as_ref()
                .is_none_or(|status| status.supported);
        let installed = self
            .system_status
            .as_ref()
            .is_some_and(|status| status.installed);
        let present = self
            .system_status
            .as_ref()
            .is_some_and(|status| status.has_entries);
        let busy = self.system_busy || self.saving;
        let status = if self.system_busy {
            "正在处理…"
        } else if !supported {
            "当前系统暂不支持"
        } else if self.system_status.is_none() {
            "尚未获取状态"
        } else if installed {
            "已添加"
        } else if present {
            "需要修复或更新程序路径"
        } else {
            "未添加"
        };
        v_flex().h_full().flex_1().min_w_0().min_h_0().child(crate::gui::component::panel::settings_header("系统设置")).child(
            v_flex()
                .id("system-settings-content")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .mx_4()
                .mb_4()
                .pt_5()
                .gap_4()
                .border_t_1()
                .border_color(crate::component::appearance::border(0xe5e7eb))
                .child(
                    crate::gui::component::panel::card(0xe5e7eb)
                        .child(
                            h_flex().items_center().gap_3().child(div().flex_1().font_weight(FontWeight::SEMIBOLD).child("Windows 右键菜单")).child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                    .bg(crate::component::appearance::background(if installed { 0xecfdf5 } else { 0xf3f4f6 }))
                                    .text_color(crate::component::appearance::foreground(if installed { 0x047857 } else { 0x6b7280 }))
                                    .child(status),
                            ),
                        )
                        .child(div().child("在文件、文件夹和文件夹空白处的右键菜单中添加“使用 GPUI Editor 打开”。"))
                        .child(
                            div()
                                .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                .text_color(crate::component::appearance::foreground(0x6b7280))
                                .child("兼容 Windows 10 / 11，仅对当前用户生效，无需管理员权限。Windows 10 直接显示；Windows 11 位于“显示更多选项”。"),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("system-context-menu-install")
                                        .primary()
                                        .label(if present && !installed { "修复右键菜单" } else { "添加到右键菜单" })
                                        .disabled(busy || !supported || installed || self.system_status.is_none())
                                        .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Install, cx))),
                                )
                                .child(
                                    Button::new("system-context-menu-remove")
                                        .label("移除")
                                        .disabled(busy || !supported || !present)
                                        .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Remove, cx))),
                                )
                                .child(
                                    Button::new("system-context-menu-refresh")
                                        .label("刷新状态")
                                        .disabled(busy)
                                        .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Refresh, cx))),
                                ),
                        )
                        .child(
                            div()
                                .text_size(crate::gui::component::appearance::ui_text_size(12.))
                                .text_color(crate::component::appearance::foreground(0x6b7280))
                                .child("添加或移除后立即生效，不受底部“应用 / 取消”影响。移动程序位置后，可点击修复更新菜单。"),
                        )
                        .when_some(self.system_message.clone(), |content, message| {
                            content.child(div().text_color(crate::component::appearance::foreground(0x047857)).child(message))
                        })
                        .when_some(self.system_error.clone(), |content, error| {
                            content.child(div().text_color(crate::component::appearance::foreground(0xd44352)).child(error))
                        }),
                )
                .child(self.render_file_associations(cx)),
        )
    }

    fn render_file_associations(&self, cx: &mut Context<Self>) -> Div {
        let is_default = |extension: &str| {
            self.association_status.as_ref().is_some_and(|status| {
                status
                    .default_extensions
                    .iter()
                    .any(|current| current == extension)
            })
        };
        let custom_extensions = self
            .association_status
            .as_ref()
            .map(|status| {
                status
                    .extensions
                    .iter()
                    .filter(|extension| {
                        !crate::domain::settings::EDITOR_FILE_EXTENSIONS
                            .contains(&extension.as_str())
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let busy = self.system_busy || self.saving || !cfg!(windows);
        let registered = self
            .association_status
            .as_ref()
            .is_some_and(|status| status.registered);
        let label = if self.system_busy {
            "正在处理…"
        } else if !cfg!(windows) {
            "当前系统暂不支持"
        } else if self.association_status.is_none() {
            "尚未获取状态"
        } else if registered {
            "文件类型已注册"
        } else {
            "文件类型未注册"
        };
        crate::gui::component::panel::card(0xe5e7eb)
            .child(h_flex().items_center().gap_3()
                .child(div().flex_1().font_weight(FontWeight::SEMIBOLD).child("默认文件打开方式"))
                .child(div().px_2().py_1().rounded_md()
                    .text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .bg(crate::component::appearance::background(if registered { 0xecfdf5 } else { 0xf3f4f6 }))
                    .text_color(crate::component::appearance::foreground(if registered { 0x047857 } else { 0x6b7280 }))
                    .child(label)))
            .child(div().child("将 GPUI Editor 注册为文本和源代码文件的可选编辑器，在 Windows 设置中选择默认打开方式。"))
            .child(v_flex().gap_2()
                .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(0x6b7280)).child("预置文件类型"))
                .child(h_flex().flex_wrap().gap_2().children(
                    crate::domain::settings::EDITOR_FILE_EXTENSIONS.iter().map(|extension| extension_tag(extension, is_default(extension))))))
            .when(!custom_extensions.is_empty(), |card| card.child(v_flex().gap_2()
                .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(0x6b7280)).child("已添加的自定义类型"))
                .child(h_flex().flex_wrap().gap_2().children(custom_extensions.iter().map(|extension| extension_tag(extension, is_default(extension)))))))
            .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.))
                .text_color(crate::component::appearance::foreground(0x6b7280))
                .child("高亮表示已由 GPUI Editor 默认打开；其他类型未设为默认或尚未确认。在 Windows 中修改后，请点击“刷新状态”。"))
            .child(v_flex().gap_2()
                .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.)).child("添加自定义类型"))
                .child(h_flex().items_center().gap_2()
                    .child(div().flex_1().min_w_0().child(Input::new(&self.association_input).disabled(busy)))
                    .child(Button::new("system-file-associations-add").label("添加类型")
                        .disabled(busy || self.association_input.read(cx).value().trim().is_empty())
                        .on_click(cx.listener(|this, _, _, cx| this.configure_file_associations(false, cx)))))
                .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.))
                    .text_color(crate::component::appearance::foreground(0x6b7280))
                    .child("多个扩展名使用 ; 分隔，例如 .vue;.c;.cpp。默认类型会保留，重复项自动合并。")))
            .when(cfg!(windows), |card| card.child(div()
                .text_size(crate::gui::component::appearance::ui_text_size(12.))
                .text_color(crate::component::appearance::foreground(0x6b7280))
                .child("Windows 10 打开设置后，请滚动到底部，选择“按应用设置默认值 → GPUI Editor → 管理”。")))
            .child(h_flex().gap_2()
                .child(Button::new("system-file-associations-configure").primary()
                    .label(if registered { "打开默认应用设置" } else { "注册并配置默认应用" })
                    .disabled(self.system_busy || self.saving || !cfg!(windows))
                    .on_click(cx.listener(|this, _, _, cx| this.configure_file_associations(true, cx))))
                .child(Button::new("system-file-associations-refresh").label("刷新状态")
                    .disabled(self.system_busy || self.saving || !cfg!(windows))
                    .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Refresh, cx)))))
            .child(div().text_size(crate::gui::component::appearance::ui_text_size(12.))
                .text_color(crate::component::appearance::foreground(0x6b7280))
                .child("仅对当前用户生效，无需管理员权限。注册不代表已设为默认；请在 Windows 默认应用中选择 GPUI Editor。操作立即生效，不受“应用 / 取消”影响。移动程序后请重新注册。"))
            .when_some(self.association_message.clone(), |card, message| card.child(
                div().text_color(crate::component::appearance::foreground(0x047857)).child(message)))
            .when_some(self.association_error.clone(), |card, error| card.child(
                div().text_color(crate::component::appearance::foreground(0xd44352)).child(error)))
    }
}

fn extension_tag(extension: &str, is_default: bool) -> Div {
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(crate::component::appearance::border(if is_default {
            0xa7f3d0
        } else {
            0xe5e7eb
        }))
        .text_size(crate::gui::component::appearance::ui_text_size(12.))
        .when(is_default, |tag| {
            tag.bg(crate::component::appearance::background(0xecfdf5))
        })
        .text_color(crate::component::appearance::foreground(if is_default {
            0x047857
        } else {
            0x6b7280
        }))
        .child(extension.to_owned())
}
