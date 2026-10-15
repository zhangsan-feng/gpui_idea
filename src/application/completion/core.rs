use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, Position, Range,
    TextEdit,
};
use ropey::{LineType, Rope};
use std::collections::BTreeMap;

const SCAN_BYTES: usize = 256 * 1024;
const MAX_ITEMS: usize = 80;

fn identifier(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn position(text: &Rope, offset: usize) -> Position {
    let line = text.byte_to_line_idx(offset, LineType::LF);
    let start = text.line_to_byte_idx(line, LineType::LF);
    // GPUI 0.7.3's RopeExt::position_to_offset consumes Unicode scalar columns,
    // including for CompletionItem text edits (unlike an external LSP server).
    let character = text.slice(start..offset).chars().count();
    Position::new(line as u32, character as u32)
}

pub(super) fn collect(text: &Rope, offset: usize, language: &str) -> CompletionResponse {
    let started = std::time::Instant::now();
    if offset > text.len() || !text.is_char_boundary(offset) {
        return CompletionResponse::Array(vec![]);
    }
    let prefix_len: usize = text
        .chars_at(offset)
        .reversed()
        .take_while(|ch| identifier(*ch))
        .take(129)
        .map(char::len_utf8)
        .sum();
    if prefix_len > 128 {
        return CompletionResponse::Array(vec![]);
    }
    let start = offset - prefix_len;
    let prefix = text.slice(start..offset).to_string();
    let suffix_len: usize = text
        .chars_at(offset)
        .take_while(|ch| identifier(*ch))
        .take(129)
        .map(char::len_utf8)
        .sum();
    if suffix_len > 128 {
        return CompletionResponse::Array(vec![]);
    }
    let end = offset + suffix_len;
    let replacement = Range::new(position(text, start), position(text, end));
    let mut candidates = BTreeMap::new();
    for word in super::keywords::words(language).split_whitespace() {
        if word.starts_with(&prefix) {
            candidates.insert(word.to_owned(), (CompletionItemKind::KEYWORD, "关键字"));
        }
    }

    // Bound each request's work on large files; prefer the text around the caret.
    let mut scan_start = start.saturating_sub(SCAN_BYTES / 2);
    while !text.is_char_boundary(scan_start) {
        scan_start += 1;
    }
    let mut scan_end = (scan_start + SCAN_BYTES).min(text.len());
    while !text.is_char_boundary(scan_end) {
        scan_end -= 1;
    }
    let mut word = String::new();
    let mut word_start = scan_start;
    let mut byte = scan_start;
    let scan = text.slice(scan_start..scan_end);
    for ch in scan.chars().chain(std::iter::once(' ')) {
        if identifier(ch) {
            if word.is_empty() {
                word_start = byte;
            }
            // Skip pathological tokens without allocating a file-sized string.
            if byte - word_start < 512 {
                word.push(ch);
            }
        } else {
            if !word.is_empty()
                && byte - word_start < 512
                && !(word_start <= offset && byte >= start)
                && (word_start > scan_start || scan_start == 0)
                && (byte < scan_end || scan_end == text.len())
                && word.starts_with(&prefix)
                && word != prefix
                && !word.chars().next().is_some_and(char::is_numeric)
                && candidates.len() < 2000
            {
                let (kind, detail) = if ch == '(' {
                    (CompletionItemKind::FUNCTION, "当前文档 · 函数")
                } else {
                    (CompletionItemKind::VARIABLE, "当前文档 · 标识符")
                };
                candidates.entry(word.clone()).or_insert((kind, detail));
            }
            word.clear();
        }
        byte += ch.len_utf8();
    }
    let mut candidates: Vec<_> = candidates.into_iter().collect();
    candidates.sort_by(|(a, (ak, _)), (b, (bk, _))| {
        (a != &prefix, *ak != CompletionItemKind::KEYWORD, a).cmp(&(
            b != &prefix,
            *bk != CompletionItemKind::KEYWORD,
            b,
        ))
    });
    let items: Vec<_> = candidates
        .into_iter()
        .take(MAX_ITEMS)
        .map(|(label, (kind, detail))| CompletionItem {
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range: replacement,
                new_text: label.clone(),
            })),
            label,
            filter_text: Some(prefix.clone()),
            kind: Some(kind),
            detail: Some(detail.into()),
            ..Default::default()
        })
        .collect();
    tracing::debug!(
        language,
        count = items.len(),
        elapsed_us = started.elapsed().as_micros(),
        "local code completions collected"
    );
    CompletionResponse::Array(items)
}
