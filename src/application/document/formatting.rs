use super::DocumentService;
use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::{Result, bail, ensure};
use lsp_types::{Position, TextEdit};
use std::path::Path;

impl DocumentService {
    pub(super) async fn format_before_save(
        &self,
        path: &Path,
        automatic: bool,
    ) -> Option<(u64, ropey::Rope)> {
        if !DATA_CONTEXT.settings.get().auto_format {
            return None;
        }
        let snapshot = DATA_CONTEXT.documents.snapshot(path)?;
        if snapshot.saving
            || (automatic
                && (DATA_CONTEXT.documents.autosave_paused()
                    || snapshot.save_error.is_some()
                    || !snapshot.dirty()))
        {
            return None;
        }
        let language = crate::domain::language_server::language_for_path(path)?;
        let revision = snapshot.revision;
        let result = async {
            let source = snapshot.text.clone();
            let text = tokio::task::spawn_blocking(move || source.to_string()).await?;
            let edits = INFRASTRUCTURE
                .language_servers
                .format_document(
                    snapshot.scope_root.clone(),
                    snapshot.path.clone(),
                    text.clone(),
                    language,
                )
                .await?;
            if edits.is_empty() {
                return Ok(None);
            }
            tokio::task::spawn_blocking(move || {
                apply_edits(text, edits)
                    .map(|text| Some((revision, ropey::Rope::from(text.as_str()))))
            })
            .await?
        }
        .await;
        match result {
            Ok(formatted) => formatted,
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "LSP formatting skipped; saving original document");
                None
            }
        }
    }
}

fn offset(text: &str, position: Position) -> Result<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        let Some(end) = text[start..].find('\n') else {
            bail!("格式化行号超出范围");
        };
        start += end + 1;
    }
    let line = text[start..]
        .split('\n')
        .next()
        .unwrap_or_default()
        .trim_end_matches('\r');
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units == position.character {
            return Ok(start + byte);
        }
        units += ch.len_utf16() as u32;
    }
    ensure!(units == position.character, "格式化 UTF-16 列位置无效");
    Ok(start + line.len())
}

fn apply_edits(mut text: String, edits: Vec<TextEdit>) -> Result<String> {
    let mut changes = edits
        .into_iter()
        .map(|edit| {
            let start = offset(&text, edit.range.start)?;
            let end = offset(&text, edit.range.end)?;
            ensure!(start <= end, "格式化范围无效");
            Ok((start, end, edit.new_text))
        })
        .collect::<Result<Vec<_>>>()?;
    changes.sort_by_key(|(start, end, _)| (*start, *end));
    let mut boundary = text.len();
    for (start, end, replacement) in changes.into_iter().rev() {
        ensure!(end <= boundary, "格式化返回了重叠的编辑范围");
        text.replace_range(start..end, &replacement);
        boundary = start;
    }
    Ok(text)
}
