use super::{LinkScope, OutdentEdit, core};
use anyhow::{Context, Result};
use ropey::Rope;
use std::ops::Range;

pub(crate) fn outdent_edit(text: &Rope, selection: Range<usize>) -> Option<OutdentEdit> {
    let edit = core::outdent_edit(text, selection)?;
    tracing::debug!(
        removed_bytes = edit.range.len() - edit.replacement.len(),
        "outdenting short or mixed document whitespace"
    );
    Some(edit)
}

#[cfg(windows)]
pub(crate) async fn clipboard_text_with_normalized_breaks() -> Result<Option<String>> {
    tokio::task::spawn_blocking(|| {
        let text = crate::infrastructure::clipboard::read_text()?;
        let (normalized, bare_cr) = core::normalize_pasted_line_endings(&text)?;
        tracing::debug!(
            bare_cr,
            bytes = text.len(),
            "normalizing pasted line endings"
        );
        Some(normalized)
    })
    .await
    .context("读取编辑器剪贴板任务失败")
}

pub(crate) async fn web_link_at_offset(
    buffer: Rope,
    offset: usize,
    is_markdown: bool,
) -> Result<Option<String>> {
    tokio::task::spawn_blocking(move || {
        let target = core::web_link_at_offset(&buffer, offset, is_markdown);
        tracing::debug!(
            offset,
            found = target.is_some(),
            "Editor web link lookup completed"
        );
        target
    })
    .await
    .context("网页链接解析任务失败")
}

pub(crate) async fn web_link_spans(buffer: Rope, scope: LinkScope) -> Result<Vec<Range<usize>>> {
    tokio::task::spawn_blocking(move || core::web_link_spans(&buffer, scope))
        .await
        .context("网页链接范围解析任务失败")
}
