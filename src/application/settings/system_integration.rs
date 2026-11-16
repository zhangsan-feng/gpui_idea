use super::SettingsService;
use crate::{
    data_context::DATA_CONTEXT,
    domain::settings::{ContextMenuAction, ContextMenuStatus},
    infrastructure::INFRASTRUCTURE,
};
use anyhow::Result;

impl SettingsService {
    pub(super) async fn configure_file_associations_core(
        &self,
        input: String,
        open_settings: bool,
    ) -> Result<crate::domain::settings::FileAssociationStatus> {
        use crate::domain::settings::FileAssociationAction;
        let extensions = parse_file_extensions(&input)?;
        let action = if open_settings {
            FileAssociationAction::Configure(extensions)
        } else {
            FileAssociationAction::Register(extensions)
        };
        self.file_associations_core(action).await
    }

    pub(super) async fn file_associations_core(
        &self,
        action: crate::domain::settings::FileAssociationAction,
    ) -> Result<crate::domain::settings::FileAssociationStatus> {
        let _guard = self.system_io.lock().await;
        let result = INFRASTRUCTURE
            .system_integration
            .file_associations(action.clone())
            .await;
        match &result {
            Ok(status) => DATA_CONTEXT
                .settings
                .publish_file_associations(Some(status.clone())),
            Err(error) => {
                DATA_CONTEXT.settings.publish_file_associations(None);
                tracing::error!(?action, error = %format!("{error:#}"), "file association operation failed");
            }
        }
        result
    }

    pub(super) async fn context_menu_core(
        &self,
        action: ContextMenuAction,
    ) -> Result<ContextMenuStatus> {
        let _guard = self.system_io.lock().await;
        match INFRASTRUCTURE.system_integration.context_menu(action).await {
            Ok(status) => {
                DATA_CONTEXT
                    .settings
                    .publish_context_menu(Some(status.clone()));
                Ok(status)
            }
            Err(error) => {
                DATA_CONTEXT.settings.publish_context_menu(None);
                tracing::error!(?action, error = %format!("{error:#}"), "system integration failed");
                Err(error)
            }
        }
    }
}

fn parse_file_extensions(input: &str) -> Result<Vec<String>> {
    let mut extensions = crate::domain::settings::EDITOR_FILE_EXTENSIONS
        .iter()
        .map(|extension| (*extension).to_owned())
        .collect::<Vec<_>>();
    for part in input
        .split([';', '；'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let suffix = part.strip_prefix('.').unwrap_or(part);
        anyhow::ensure!(
            !suffix.is_empty()
                && suffix.len() <= 64
                && suffix
                    .chars()
                    .all(|character| character.is_alphanumeric()
                        || matches!(character, '_' | '-' | '+')),
            "无效的文件类型“{part}”，请输入 .vue;.c;.cpp 这样的扩展名，并使用 ; 分隔"
        );
        let extension = format!(".{}", suffix.to_lowercase());
        if !extensions.contains(&extension) {
            extensions.push(extension);
        }
    }
    anyhow::ensure!(extensions.len() <= 256, "一次最多注册 256 种文件类型");
    Ok(extensions)
}
