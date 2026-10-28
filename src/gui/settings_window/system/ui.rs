use super::super::*;
use crate::domain::settings::ContextMenuAction;
use gpui_kit::{
    component::{
        Disableable as _,
        button::{Button, ButtonVariants as _},
        h_flex, v_flex,
    },
    prelude::*,
    *,
};

impl SettingsWindow {
    pub(in crate::gui) fn render_system(
        &self,
        cx: &mut Context<Self>,
    ) -> Div {
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
        v_flex().h_full().flex_1().min_w_0().min_h_0()
            .child(crate::gui::component::panel::settings_header("系统设置"))
            .child(v_flex().id("system-settings-content").flex_1().min_h_0().overflow_y_scroll()
                .mx_4().mb_4().pt_5().border_t_1().border_color(crate::component::appearance::border(0xe5e7eb))
                .child(crate::gui::component::panel::card(0xe5e7eb)
                    .child(h_flex().items_center().gap_3()
                        .child(div().flex_1().font_weight(FontWeight::SEMIBOLD).child("Windows 右键菜单"))
                        .child(div().px_2().py_1().rounded_md().text_size(px(12.))
                            .bg(crate::component::appearance::background(if installed { 0xecfdf5 } else { 0xf3f4f6 }))
                            .text_color(crate::component::appearance::foreground(if installed { 0x047857 } else { 0x6b7280 })).child(status)))
                    .child(div().child("在文件、文件夹和文件夹空白处的右键菜单中添加“使用 GPUI Editor 打开”。"))
                    .child(div().text_size(px(12.)).text_color(crate::component::appearance::foreground(0x6b7280))
                        .child("兼容 Windows 10 / 11，仅对当前用户生效，无需管理员权限。Windows 10 直接显示；Windows 11 位于“显示更多选项”。"))
                    .child(h_flex().gap_2()
                        .child(Button::new("system-context-menu-install").primary()
                            .label(if present && !installed { "修复右键菜单" } else { "添加到右键菜单" })
                            .disabled(busy || !supported || installed || self.system_status.is_none())
                            .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Install, cx))))
                        .child(Button::new("system-context-menu-remove").label("移除")
                            .disabled(busy || !supported || !present)
                            .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Remove, cx))))
                        .child(Button::new("system-context-menu-refresh").label("刷新状态").disabled(busy)
                            .on_click(cx.listener(|this, _, _, cx| this.manage_context_menu(ContextMenuAction::Refresh, cx)))))
                    .child(div().text_size(px(12.)).text_color(crate::component::appearance::foreground(0x6b7280))
                        .child("添加或移除后立即生效，不受底部“应用 / 取消”影响。移动程序位置后，可点击修复更新菜单。"))
                    .when_some(self.system_message.clone(), |content, message| content.child(div().text_color(crate::component::appearance::foreground(0x047857)).child(message)))
                    .when_some(self.system_error.clone(), |content, error| content.child(div().text_color(crate::component::appearance::foreground(0xd44352)).child(error)))))
    }
}
