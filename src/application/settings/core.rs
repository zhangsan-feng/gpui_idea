use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::settings::{EDITOR_FONT_SIZE_RANGE, GeneralSettings},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn load_core(&self) -> Result<()> {
        let _guard = self.io.lock().await;
        let mut value = INFRASTRUCTURE.settings.load().await?;
        self.normalize_ignored_directories(&mut value.ignored_directories)?;
        let defaults = crate::domain::settings::AppearanceSettings::default();
        if value.appearance.font_defaults_version < defaults.font_defaults_version {
            if value.appearance.ui_font == "Inter Variable"
                || (value.appearance.font_defaults_version == 0
                    && value.appearance.ui_font == "Microsoft YaHei UI")
            {
                value.appearance.ui_font = defaults.ui_font;
            }
            if value.appearance.font_defaults_version == 0
                && value.appearance.editor_font == "Cascadia Mono"
            {
                value.appearance.editor_font = defaults.editor_font;
            }
            value.appearance.font_defaults_version = defaults.font_defaults_version;
        }

        let fonts = INFRASTRUCTURE.settings.load_fonts().await?;
        DATA_CONTEXT.settings.publish_fonts(fonts);
        INFRASTRUCTURE.proxy.validate(&value.lsp_proxy)?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        INFRASTRUCTURE
            .filesystem
            .set_ignored_directories(value.ignored_directories.clone());
        DATA_CONTEXT.settings.publish_general(value);
        Ok(())
    }

    pub(super) async fn save_core(
        &self,
        mut value: GeneralSettings,
        editor_size_changed: bool,
    ) -> Result<()> {
        let _guard = self.io.lock().await;
        let pending_size = DATA_CONTEXT.settings.pending_editor_font_size();
        self.normalize_ignored_directories(&mut value.ignored_directories)?;
        let filter_changed = DATA_CONTEXT.settings.snapshot().general.ignored_directories
            != value.ignored_directories;
        value.lsp_proxy.url = value.lsp_proxy.url.trim().to_owned();
        value.appearance.ui_font = value.appearance.ui_font.trim().to_owned();
        value.appearance.editor_font = value.appearance.editor_font.trim().to_owned();
        anyhow::ensure!(
            !value.appearance.ui_font.is_empty() && !value.appearance.editor_font.is_empty(),
            "字体名称不能为空"
        );
        anyhow::ensure!(
            EDITOR_FONT_SIZE_RANGE.contains(&value.appearance.editor_size),
            "编辑器字号须在 9–36 之间"
        );
        anyhow::ensure!(
            (10..=24).contains(&value.appearance.ui_size),
            "窗口字号须在 10–24 之间"
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
        let value = INFRASTRUCTURE
            .settings
            .save(value, !editor_size_changed)
            .await?;
        INFRASTRUCTURE.proxy.set_lsp(value.lsp_proxy.clone());
        tracing::info!(
            auto_format = value.auto_format,
            soft_wrap = value.soft_wrap,
            "general settings saved"
        );
        tracing::info!(ui_font = %value.appearance.ui_font, editor_font = %value.appearance.editor_font,
            ui_size = value.appearance.ui_size, editor_size = value.appearance.editor_size, dark = value.appearance.theme == crate::domain::settings::ColorTheme::Dark,
            "appearance settings saved");
        INFRASTRUCTURE
            .filesystem
            .set_ignored_directories(value.ignored_directories.clone());
        DATA_CONTEXT.settings.publish_general(value);
        if editor_size_changed {
            if let Some(size) = pending_size {
                DATA_CONTEXT.settings.clear_pending_editor_font_size(size);
            }
        }
        if filter_changed && DATA_CONTEXT.has_project() {
            crate::application::APPLICATION
                .file_watch
                .refresh_data_context(&crate::event_bus::FileChanges {
                    rescan: true,
                    event_count: 1,
                    ..Default::default()
                })
                .await;
        }
        Ok(())
    }

    pub(super) fn adjust_editor_font_size_core(&self, step: i16) -> bool {
        let current = DATA_CONTEXT.settings.editor_font_size();
        let size = current.saturating_add_signed(step).clamp(
            *EDITOR_FONT_SIZE_RANGE.start(),
            *EDITOR_FONT_SIZE_RANGE.end(),
        );
        if size == current {
            return false;
        }
        DATA_CONTEXT.settings.stage_editor_font_size(size);
        true
    }

    pub(super) async fn flush_editor_font_size_core(&self) -> Result<()> {
        let _guard = self.io.lock().await;
        let Some(size) = DATA_CONTEXT.settings.pending_editor_font_size() else {
            return Ok(());
        };
        INFRASTRUCTURE.settings.save_editor_font_size(size).await?;
        DATA_CONTEXT.settings.publish_editor_font_size(size);
        DATA_CONTEXT.settings.clear_pending_editor_font_size(size);
        tracing::info!(editor_size = size, "editor font size saved");
        Ok(())
    }
}
