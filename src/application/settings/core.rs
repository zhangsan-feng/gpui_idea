use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT, domain::settings::GeneralSettings, infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn load_core(&self) -> Result<()> {
        let _guard = self.io.lock().await;
        let value = INFRASTRUCTURE.settings.load().await?;
        INFRASTRUCTURE.proxy.validate(&value.lsp_proxy)?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        DATA_CONTEXT.settings.publish_general(value);
        Ok(())
    }

    pub(super) async fn save_core(&self, mut value: GeneralSettings) -> Result<()> {
        let _guard = self.io.lock().await;
        value.lsp_proxy.url = value.lsp_proxy.url.trim().to_owned();
        value.appearance.ui_font = value.appearance.ui_font.trim().to_owned();
        value.appearance.editor_font = value.appearance.editor_font.trim().to_owned();
        anyhow::ensure!(
            !value.appearance.ui_font.is_empty() && !value.appearance.editor_font.is_empty(),
            "字体名称不能为空"
        );
        anyhow::ensure!(
            (9..=36).contains(&value.appearance.editor_size),
            "编辑器字号须在 9–36 之间"
        );
        INFRASTRUCTURE.proxy.validate(&value.lsp_proxy)?;
        if let Some(id) = &value.appearance.selected_theme {
            let theme = value
                .appearance
                .imported_themes
                .iter()
                .find(|theme| &theme.id == id)
                .ok_or_else(|| anyhow::anyhow!("选择的主题不存在，请重新选择"))?;
            value.appearance.theme = if theme.dark {
                crate::domain::settings::ColorTheme::Dark
            } else {
                crate::domain::settings::ColorTheme::Light
            };
        }
        INFRASTRUCTURE.settings.save(value.clone()).await?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        tracing::info!(
            auto_format = value.auto_format,
            soft_wrap = value.soft_wrap,
            "general settings saved"
        );
        tracing::info!(ui_font = %value.appearance.ui_font, editor_font = %value.appearance.editor_font,
            editor_size = value.appearance.editor_size, dark = value.appearance.theme == crate::domain::settings::ColorTheme::Dark,
            "appearance settings saved");
        DATA_CONTEXT.settings.publish_general(value);
        Ok(())
    }
}
