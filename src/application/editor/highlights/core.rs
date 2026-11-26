use crate::{data_context::DATA_CONTEXT, infrastructure::INFRASTRUCTURE};
use anyhow::Result;
use lsp_types::Position;
use ropey::{LineType, Rope};
use std::{ops::Range, path::PathBuf};
use tokio::sync::watch;

fn identifier(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '_' | '$')
}

fn cancelled(cancel: &watch::Receiver<bool>) -> bool {
    *cancel.borrow() || cancel.has_changed().is_err()
}

fn occurrences(
    text: &str,
    selection: Range<usize>,
    cancel: &watch::Receiver<bool>,
) -> Vec<Range<usize>> {
    let Some(word) = text.get(selection.clone()) else {
        return vec![];
    };
    // Only a complete identifier is a variable selection; multi-line text and
    // partial words should not flood the document with unrelated matches.
    if word.is_empty()
        || word.len() > 256
        || !word.chars().all(identifier)
        || word.chars().next().is_some_and(char::is_numeric)
        || text[..selection.start]
            .chars()
            .next_back()
            .is_some_and(identifier)
        || text[selection.end..].chars().next().is_some_and(identifier)
    {
        return vec![];
    }
    let mut ranges = Vec::new();
    for (start, _) in text.match_indices(word) {
        if cancelled(cancel) {
            break;
        }
        let end = start + word.len();
        if !text[..start].chars().next_back().is_some_and(identifier)
            && !text[end..].chars().next().is_some_and(identifier)
        {
            ranges.push(start..end);
            if ranges.len() == 10_000 {
                break;
            }
        }
    }
    ranges
}

fn byte_offset(text: &Rope, position: Position) -> Option<usize> {
    if position.line as usize >= text.len_lines(LineType::LF) {
        return None;
    }
    let start = text.line_to_byte_idx(position.line as usize, LineType::LF);
    let mut units = 0;
    let mut offset = start;
    for ch in text.chars_at(start) {
        if units == position.character {
            return Some(offset);
        }
        if ch == '\n' || ch == '\r' {
            return None;
        }
        units += ch.len_utf16() as u32;
        offset += ch.len_utf8();
        if units > position.character {
            return None;
        }
    }
    (units == position.character).then_some(offset)
}

pub(super) async fn resolve(
    path: PathBuf,
    text: Rope,
    selection: Range<usize>,
    cursor: usize,
    cancel: watch::Receiver<bool>,
) -> Result<Vec<Range<usize>>> {
    if cancelled(&cancel) {
        return Ok(vec![]);
    }
    if !selection.is_empty() {
        return Ok(tokio::task::spawn_blocking(move || {
            occurrences(&text.to_string(), selection, &cancel)
        })
        .await?);
    }
    let Some(language) = crate::domain::language_server::language_for_path(&path) else {
        return Ok(vec![]);
    };
    if cursor > text.len()
        || !text.is_char_boundary(cursor)
        || !text.chars_at(cursor).next().is_some_and(identifier)
            && !text
                .chars_at(cursor)
                .reversed()
                .next()
                .is_some_and(identifier)
    {
        return Ok(vec![]);
    }
    let buffer = text.clone();
    let (position, source) = tokio::task::spawn_blocking(move || {
        let line = buffer.byte_to_line_idx(cursor, LineType::LF);
        let start = buffer.line_to_byte_idx(line, LineType::LF);
        let units = buffer
            .slice(start..cursor)
            .chars()
            .map(char::len_utf16)
            .sum::<usize>();
        (Position::new(line as u32, units as u32), buffer.to_string())
    })
    .await?;
    let ranges = INFRASTRUCTURE
        .language_servers
        .document_highlights(
            DATA_CONTEXT.project_root().to_owned(),
            path,
            source,
            position,
            language,
            cancel.clone(),
        )
        .await?;
    // Convert the server's UTF-16 positions against the exact requested buffer.
    Ok(tokio::task::spawn_blocking(move || {
        ranges
            .into_iter()
            .take(10_000)
            .filter_map(|highlight| {
                if cancelled(&cancel) {
                    return None;
                }
                let start = byte_offset(&text, highlight.range.start)?;
                let end = byte_offset(&text, highlight.range.end)?;
                (start < end).then_some(start..end)
            })
            .collect()
    })
    .await?)
}
