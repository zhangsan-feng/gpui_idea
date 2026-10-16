use anyhow::{Context as _, Result};
use lsp_types::CompletionResponse;
use ropey::Rope;

pub(crate) async fn complete(
    path: std::path::PathBuf,
    text: Rope,
    offset: usize,
    language: &'static str,
    cancelled: tokio::sync::watch::Receiver<bool>,
) -> Result<CompletionResponse> {
    if crate::domain::language_server::family(language).is_some() {
        let response =
            super::semantic::complete(path, text.clone(), offset, language, cancelled.clone())
                .await;
        match response {
            Ok(response) => return Ok(response),
            Err(error) => {
                if *cancelled.borrow() || cancelled.has_changed().is_err() {
                    return Err(error);
                }
                tracing::debug!(language, %error, "LSP completion unavailable; using document completions");
            }
        }
    }
    tokio::task::spawn_blocking(move || super::core::collect(&text, offset, language))
        .await
        .context("代码提示后台任务失败")
}
