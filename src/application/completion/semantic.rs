use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::Result;
use lsp_types::{CompletionResponse, CompletionTextEdit, Position, Range};
use ropey::{LineType, Rope};
use std::path::PathBuf;
use tokio::sync::watch;

pub(super) async fn complete(
    path: PathBuf,
    text: Rope,
    offset: usize,
    language: &'static str,
    cancelled: watch::Receiver<bool>,
) -> Result<CompletionResponse> {
    let snapshots = DATA_CONTEXT.documents.snapshots();
    let current = path.clone();
    let language = crate::domain::language_server::language_for_path(&path).unwrap_or(language);
    let (text, source, position, prefix, overlays) =
        tokio::task::spawn_blocking(move || -> Result<_> {
            anyhow::ensure!(
                offset <= text.len() && text.is_char_boundary(offset),
                "补全位置无效"
            );
            let line = text.byte_to_line_idx(offset, LineType::LF);
            let start = text.line_to_byte_idx(line, LineType::LF);
            let position = Position::new(
                line as u32,
                text.slice(start..offset)
                    .chars()
                    .map(char::len_utf16)
                    .sum::<usize>() as u32,
            );
            let prefix: String = text
                .chars_at(offset)
                .reversed()
                .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let overlays = snapshots
                .into_iter()
                .filter(|snapshot| {
                    snapshot.path != current
                        && crate::domain::language_server::language_for_path(&snapshot.path)
                            .and_then(crate::domain::language_server::family)
                            == crate::domain::language_server::family(language)
                })
                .map(|snapshot| (snapshot.path.clone(), snapshot.text.to_string()))
                .collect();
            let source = text.to_string();
            Ok((text, source, position, prefix, overlays))
        })
        .await??;
    let response = INFRASTRUCTURE
        .language_servers
        .completions(
            DATA_CONTEXT.project_root().to_owned(),
            path,
            source,
            position,
            language,
            overlays,
            cancelled,
        )
        .await?;
    // The editor consumes scalar columns; LSP negotiated UTF-16. Convert every
    // returned edit against the exact snapshot used for this request.
    tokio::task::spawn_blocking(move || {
        let mut items = match response {
            CompletionResponse::Array(items) => items,
            CompletionResponse::List(list) => list.items,
        };
        let prefix_bytes = prefix.len();
        let prefix = prefix.to_lowercase();
        items.retain_mut(|item| {
            if item
                .additional_text_edits
                .as_ref()
                .is_some_and(|edits| !edits.is_empty())
            {
                return false;
            }
            if item.insert_text_format == Some(lsp_types::InsertTextFormat::SNIPPET) {
                let source = match &item.text_edit {
                    Some(CompletionTextEdit::Edit(edit)) => &edit.new_text,
                    Some(CompletionTextEdit::InsertAndReplace(edit)) => &edit.new_text,
                    None => item.insert_text.as_ref().unwrap_or(&item.label),
                };
                let Some(plain) = super::snippet::plain_text(source) else {
                    return false;
                };
                match &mut item.text_edit {
                    Some(CompletionTextEdit::Edit(edit)) => edit.new_text = plain,
                    Some(CompletionTextEdit::InsertAndReplace(edit)) => edit.new_text = plain,
                    None => item.insert_text = Some(plain),
                }
                item.insert_text_format = Some(lsp_types::InsertTextFormat::PLAIN_TEXT);
            }
            let candidate = item
                .filter_text
                .as_deref()
                .unwrap_or(&item.label)
                .to_lowercase();
            let mut chars = candidate.chars();
            prefix
                .chars()
                .all(|ch| chars.by_ref().any(|next| next == ch))
        });
        items.sort_by(|a, b| {
            let rank = |item: &lsp_types::CompletionItem| {
                !item
                    .filter_text
                    .as_deref()
                    .unwrap_or(&item.label)
                    .to_lowercase()
                    .starts_with(&prefix)
            };
            (
                rank(a),
                a.sort_text.as_deref().unwrap_or(&a.label),
                &a.label,
            )
                .cmp(&(
                    rank(b),
                    b.sort_text.as_deref().unwrap_or(&b.label),
                    &b.label,
                ))
        });
        items.truncate(100);
        for item in &mut items {
            match &mut item.text_edit {
                Some(CompletionTextEdit::Edit(edit)) => convert_range(&text, &mut edit.range),
                Some(CompletionTextEdit::InsertAndReplace(edit)) => {
                    convert_range(&text, &mut edit.insert);
                    convert_range(&text, &mut edit.replace);
                }
                None => {
                    let start = offset - prefix_bytes;
                    let scalar = |byte| {
                        let line = text.byte_to_line_idx(byte, LineType::LF);
                        let start = text.line_to_byte_idx(line, LineType::LF);
                        Position::new(line as u32, text.slice(start..byte).chars().count() as u32)
                    };
                    item.text_edit = Some(CompletionTextEdit::Edit(lsp_types::TextEdit {
                        range: Range::new(scalar(start), scalar(offset)),
                        new_text: item
                            .insert_text
                            .clone()
                            .unwrap_or_else(|| item.label.clone()),
                    }));
                }
            }
        }
        tracing::debug!(count = items.len(), "semantic code completions collected");
        Ok(CompletionResponse::Array(items))
    })
    .await?
}

fn convert_range(text: &Rope, range: &mut Range) {
    for position in [&mut range.start, &mut range.end] {
        let line = (position.line as usize).min(text.byte_to_line_idx(text.len(), LineType::LF));
        let start = text.line_to_byte_idx(line, LineType::LF);
        let mut units = 0;
        let mut scalars = 0;
        for ch in text.chars_at(start).take_while(|ch| *ch != '\n') {
            if units + ch.len_utf16() > position.character as usize {
                break;
            }
            units += ch.len_utf16();
            scalars += 1;
        }
        *position = Position::new(line as u32, scalars);
    }
}
