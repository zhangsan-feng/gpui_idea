use super::super::SettingsWindow;

use gpui_kit::Context;

impl SettingsWindow {
    pub(super) fn reset_font(&mut self, editor: bool, cx: &mut Context<Self>) {
        if self.saving {
            return;
        }
        let defaults = crate::domain::settings::AppearanceSettings::default();
        if editor {
            self.draft.appearance.editor_font = defaults.editor_font;
            self.draft.appearance.editor_size = defaults.editor_size;
        } else {
            self.draft.appearance.ui_font = defaults.ui_font;
            self.draft.appearance.ui_size = defaults.ui_size;
        }
        self.error = None;
        cx.notify();
    }
}
