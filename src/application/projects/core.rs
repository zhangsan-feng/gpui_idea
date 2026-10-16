use crate::{application::APPLICATION, domain::history::OpenKind, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail};
use std::path::PathBuf;
pub(super) async fn open(path: PathBuf) -> Result<()> {
    let root = INFRASTRUCTURE.projects.validate(path).await?;
    let current = crate::data_context::DATA_CONTEXT.project_root();
    if root
        .to_string_lossy()
        .trim_start_matches("\\\\?\\")
        .eq_ignore_ascii_case(current.to_string_lossy().trim_start_matches("\\\\?\\"))
    {
        bail!("项目已在当前工作区打开");
    }
    INFRASTRUCTURE.projects.launch(root.clone()).await?;
    if let Err(error) = APPLICATION
        .history
        .record_opened(OpenKind::Project, root)
        .await
    {
        tracing::warn!(%error, "project launched but recent history could not be saved");
    }
    Ok(())
}
pub(super) async fn create(
    mut request: crate::domain::project_creation::CreateProject,
) -> Result<PathBuf> {
    use crate::domain::project_creation::ProjectLanguage;
    use anyhow::{Context as _, ensure};
    ensure!(request.location.is_absolute(), "请选择项目的完整路径");
    let name = request
        .location
        .file_name()
        .and_then(|name| name.to_str())
        .context("项目目录名称无效")?;
    let name = APPLICATION.project.validate_file_name(name)?;
    if request.language == ProjectLanguage::Rust {
        ensure!(
            name.bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'_' | b'-'))
                && name.as_bytes()[0].is_ascii_alphabetic(),
            "Rust 项目名称须以英文字母开头，且仅包含字母、数字、下划线或连字符"
        );
    }
    if request.language == ProjectLanguage::Go {
        request.module = request.module.trim().to_owned();
        if request.module.is_empty() {
            request.module = name.clone();
        }
        ensure!(
            !request.module.starts_with('-') && !request.module.chars().any(char::is_whitespace),
            "Go Module 路径不能包含空白或以 - 开头"
        );
    } else {
        request.module.clear();
    }
    if !matches!(
        request.language,
        ProjectLanguage::Rust | ProjectLanguage::Go
    ) {
        request.environment.clear();
    }
    let mut environment = Vec::new();
    for assignment in request
        .environment
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let (key, value) = assignment
            .split_once('=')
            .context("环境变量格式应为 KEY=value，多个变量用分号分隔")?;
        let key = key.trim();
        ensure!(
            !key.is_empty() && !key.contains('\0') && !value.contains('\0'),
            "环境变量名称或值无效"
        );
        environment.push((key.to_owned(), value.to_owned()));
    }
    let root = INFRASTRUCTURE
        .projects
        .create(request, name, environment)
        .await?;
    if let Err(error) = APPLICATION
        .history
        .record_opened(OpenKind::Project, root.clone())
        .await
    {
        tracing::warn!(%error, "project created but recent history could not be saved");
    }
    Ok(root)
}
