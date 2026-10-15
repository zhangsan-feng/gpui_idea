use anyhow::Result;
use ropey::Rope;
use std::path::PathBuf;

pub(crate) async fn resolve(
    path: PathBuf,
    text: Rope,
    offset: usize,
    language: &'static str,
    cancelled: tokio::sync::watch::Receiver<bool>,
) -> Result<crate::domain::definition::DefinitionResult> {
    super::core::resolve(path, text, offset, language, cancelled).await
}
