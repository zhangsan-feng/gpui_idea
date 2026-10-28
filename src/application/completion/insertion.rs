use lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, Position};
use ropey::{LineType, Rope};
use std::ops::Range;

fn set_text(item: &mut CompletionItem, text: String) {
    match &mut item.text_edit {
        Some(CompletionTextEdit::Edit(edit)) => edit.new_text = text,
        Some(CompletionTextEdit::InsertAndReplace(edit)) => edit.new_text = text,
        None => item.insert_text = Some(text),
    }
}

/// Completion ranges have already been converted to the editor's scalar columns.
fn offset(text: &Rope, position: Position) -> Option<usize> {
    let last_line = text.byte_to_line_idx(text.len(), LineType::LF);
    if position.line as usize > last_line {
        return None;
    }
    let start = text.line_to_byte_idx(position.line as usize, LineType::LF);
    let mut end = start;
    let mut remaining = position.character;
    for ch in text.chars_at(start).take_while(|ch| *ch != '\n') {
        if remaining == 0 {
            break;
        }
        end += ch.len_utf8();
        remaining -= 1;
    }
    (remaining == 0).then_some(end)
}

pub(super) fn prepare(
    mut item: CompletionItem,
    text: &Rope,
    cursor: usize,
) -> (CompletionItem, Option<Range<usize>>) {
    let (start, end) = match &item.text_edit {
        Some(CompletionTextEdit::Edit(edit)) => {
            (offset(text, edit.range.start), offset(text, edit.range.end))
        }
        Some(CompletionTextEdit::InsertAndReplace(edit)) => (
            offset(text, edit.replace.start),
            offset(text, edit.replace.end),
        ),
        None => (Some(cursor), Some(cursor)),
    };
    let (Some(start), Some(end)) = (start, end) else {
        return (item, None);
    };
    let mut insertion = match &item.text_edit {
        Some(CompletionTextEdit::Edit(edit)) => edit.new_text.clone(),
        Some(CompletionTextEdit::InsertAndReplace(edit)) => edit.new_text.clone(),
        None => item
            .insert_text
            .clone()
            .unwrap_or_else(|| item.label.clone()),
    };
    if item.insert_text_format == Some(lsp_types::InsertTextFormat::SNIPPET) {
        let Some((mut plain, mut selection)) = super::snippet::parse(&insertion) else {
            return (item, None);
        };
        // Snippet indentation is relative to the insertion line.
        if item.insert_text_mode != Some(lsp_types::InsertTextMode::AS_IS) {
            let line = text.byte_to_line_idx(start, LineType::LF);
            let line_start = text.line_to_byte_idx(line, LineType::LF);
            let indent: String = text
                .slice(line_start..start)
                .chars()
                .take_while(|ch| matches!(ch, ' ' | '\t'))
                .collect();
            if !indent.is_empty() && plain.contains('\n') {
                if let Some(range) = &mut selection {
                    let added =
                        |end| plain[..end].bytes().filter(|ch| *ch == b'\n').count() * indent.len();
                    let end_added = added(range.end);
                    range.start += added(range.start);
                    range.end += end_added;
                }
                plain = plain.replace('\n', &format!("\n{indent}"));
            }
        }
        let whitespace: usize = text
            .chars_at(end)
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .map(char::len_utf8)
            .sum();
        if plain.ends_with("()") && text.chars_at(end + whitespace).next() == Some('(') {
            plain.truncate(plain.len() - 2);
            let caret = plain.len() + whitespace + 1;
            selection = Some(caret..caret);
        }
        insertion = plain;
        set_text(&mut item, insertion.clone());
        item.insert_text_format = Some(lsp_types::InsertTextFormat::PLAIN_TEXT);
        if let Some(selection) = selection {
            tracing::debug!(kind = ?item.kind, "contextual completion snippet expanded");
            return (item, Some(start + selection.start..start + selection.end));
        }
    }
    if !matches!(item.kind, Some(kind) if kind == CompletionItemKind::FUNCTION
        || kind == CompletionItemKind::METHOD || kind == CompletionItemKind::CONSTRUCTOR)
    {
        return (item, None);
    }
    // Preserve server-provided argument text and place the caret at its start.
    if let Some(paren) = insertion.find('(') {
        let caret = start + paren + 1;
        return (item, Some(caret..caret));
    }
    if insertion.is_empty() {
        return (item, None);
    }
    let whitespace: usize = text
        .chars_at(end)
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .map(char::len_utf8)
        .sum();
    if text.chars_at(end + whitespace).next() == Some('(') {
        let caret = start + insertion.len() + whitespace + 1;
        return (item, Some(caret..caret));
    }
    let caret = start + insertion.len() + 1;
    insertion.push_str("()");
    set_text(&mut item, insertion);
    tracing::debug!(kind = ?item.kind, "callable completion parentheses added");
    (item, Some(caret..caret))
}
