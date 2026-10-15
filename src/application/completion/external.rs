use anyhow::{Context as _, Result};
use lsp_types::CompletionResponse;
use ropey::Rope;

pub(crate) async fn complete(
    text: Rope,
    offset: usize,
    language: &'static str,
) -> Result<CompletionResponse> {
    tokio::task::spawn_blocking(move || super::core::collect(&text, offset, language))
        .await
        .context("代码提示后台任务失败")
}
