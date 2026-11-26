use super::{DOCUMENT_TAB_SIZE, LinkScope, OutdentEdit};
use markdown::{ParseOptions, mdast::Node};
use regex::Regex;
use ropey::{LineType, Rope};
use std::collections::HashMap;
use std::ops::Range;
use std::sync::LazyLock;
use url::Url;

pub(super) fn outdent_edit(text: &Rope, selection: Range<usize>) -> Option<OutdentEdit> {
    let lines = LineType::LF;
    let first_row = text.byte_to_line_idx(selection.start, lines);
    let last_row = text.byte_to_line_idx(selection.end, lines);
    let mut cuts = Vec::new();
    let mut requires_fallback = false;
    for row in first_row..=last_row {
        let start = text.line_to_byte_idx(row, lines);
        let mut columns = 0;
        let mut bytes = 0;
        let mut only_ascii_spaces = true;
        for ch in text.line(row, lines).chars() {
            let width = match ch {
                '\t' => DOCUMENT_TAB_SIZE - columns,
                ' '
                | '\u{00a0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200a}'
                | '\u{202f}'
                | '\u{205f}' => 1,
                '\u{3000}' => 2,
                _ => break,
            };
            if columns + width > DOCUMENT_TAB_SIZE {
                break;
            }
            columns += width;
            bytes += ch.len_utf8();
            only_ascii_spaces &= ch == ' ';
            if columns == DOCUMENT_TAB_SIZE {
                break;
            }
        }
        if bytes > 0 {
            requires_fallback |= !only_ascii_spaces || columns < DOCUMENT_TAB_SIZE;
            cuts.push(start..start + bytes);
        }
    }

    if !requires_fallback {
        return None;
    }
    let range = cuts.first()?.start..cuts.last()?.end;
    let map_offset = |offset: usize| {
        offset
            - cuts
                .iter()
                .map(|cut| cut.end.min(offset) - cut.start.min(offset))
                .sum::<usize>()
    };
    let mut replacement = String::new();
    let mut start = range.start;
    for cut in &cuts {
        for chunk in text.slice(start..cut.start).chunks() {
            replacement.push_str(chunk);
        }
        start = cut.end;
    }
    Some(OutdentEdit {
        range,
        replacement,
        selection: map_offset(selection.start)..map_offset(selection.end),
    })
}

#[cfg(windows)]
pub(super) fn normalize_pasted_line_endings(text: &str) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let bare_cr = bytes
        .iter()
        .enumerate()
        .filter(|&(index, byte)| *byte == b'\r' && bytes.get(index + 1) != Some(&b'\n'))
        .count();

    // DirectWrite treats standalone CR as a break inside an LF-indexed row.
    // Canonicalize each break before layout, counting CRLF as one break.
    (bare_cr > 0).then(|| (text.replace("\r\n", "\n").replace('\r', "\n"), bare_cr))
}

fn reference_key(identifier: &str) -> String {
    identifier
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn definitions(node: &Node, targets: &mut HashMap<String, String>) {
    if let Node::Definition(definition) = node {
        targets
            .entry(reference_key(&definition.identifier))
            .or_insert_with(|| definition.url.clone());
    }
    for child in node.children().into_iter().flatten() {
        definitions(child, targets);
    }
}

fn destination<'a>(
    node: &'a Node,
    offset: usize,
    targets: &'a HashMap<String, String>,
) -> Option<&'a str> {
    let position = node.position()?;
    if offset < position.start.offset || offset > position.end.offset {
        return None;
    }
    link_target(node, targets).or_else(|| {
        node.children()?
            .iter()
            .find_map(|child| destination(child, offset, targets))
    })
}

fn link_target<'a>(node: &'a Node, targets: &'a HashMap<String, String>) -> Option<&'a str> {
    match node {
        Node::Link(link) => Some(&link.url),
        Node::LinkReference(link) => targets
            .get(&reference_key(&link.identifier))
            .map(String::as_str),
        _ => None,
    }
}

fn parse_web_url(target: &str) -> Option<Url> {
    let url = Url::parse(target).ok()?;
    (matches!(url.scheme(), "http" | "https") && url.host_str().is_some()).then_some(url)
}

fn markdown_url_at_offset(source: &str, offset: usize) -> Option<Url> {
    let ast = markdown::to_mdast(source, &ParseOptions::gfm()).ok()?;
    let mut targets = HashMap::new();
    definitions(&ast, &mut targets);
    parse_web_url(destination(&ast, offset, &targets)?)
}

fn trim_url_end(mut url: &str) -> &str {
    // Remove surrounding punctuation while keeping balanced brackets in the URL,
    // such as the ones used in Wikipedia paths and IPv6 hosts.
    let mut pairs = [('(', ')', 0), ('[', ']', 0), ('{', '}', 0)];
    for (open, close, balance) in &mut pairs {
        *balance = url.matches(*open).count() as isize - url.matches(*close).count() as isize;
    }
    while let Some(last) = url.chars().next_back() {
        if let Some((_, _, balance)) = pairs
            .iter_mut()
            .find(|(_, close, balance)| *close == last && *balance < 0)
        {
            *balance += 1;
        } else if !matches!(
            last,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '。'
                | '，'
                | '；'
                | '：'
                | '！'
                | '？'
                | '）'
                | '】'
        ) {
            break;
        }
        url = &url[..url.len() - last.len_utf8()];
    }
    url
}

fn plain_links(source: &str) -> impl Iterator<Item = (Range<usize>, &str)> {
    static WEB_URL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?i)(?:https?://|www\.)[^\s<>"'\\\x60]+"#).expect("valid web URL regex")
    });
    WEB_URL.find_iter(source).filter_map(move |candidate| {
        let start = candidate.start();
        if source[..start]
            .ends_with(|ch: char| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '@'))
        {
            return None;
        }
        let address = trim_url_end(candidate.as_str());
        Some((start..start + address.len(), address))
    })
}

fn parse_plain_url(address: &str) -> Option<Url> {
    if address
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("www."))
    {
        return parse_web_url(&format!("https://{address}"));
    }
    parse_web_url(address)
}

pub(super) fn web_link_at_offset(
    buffer: &Rope,
    offset: usize,
    is_markdown: bool,
) -> Option<String> {
    if offset > buffer.len() || !buffer.is_char_boundary(offset) {
        return None;
    }
    if is_markdown && let Some(url) = markdown_url_at_offset(&buffer.to_string(), offset) {
        return Some(url.into());
    }
    let line = buffer.byte_to_line_idx(offset, LineType::LF);
    let start = buffer.line_to_byte_idx(line, LineType::LF);
    let line = buffer.line(line, LineType::LF).to_string();
    let offset = offset - start;
    plain_links(&line)
        .take_while(|(span, _)| span.start <= offset)
        .find(|(span, _)| offset <= span.end)
        .and_then(|(_, address)| parse_plain_url(address))
        .map(Url::into)
}

fn markdown_link_spans(
    node: &Node,
    targets: &HashMap<String, String>,
    spans: &mut Vec<Range<usize>>,
) {
    if let Some(position) = node.position()
        && link_target(node, targets).and_then(parse_web_url).is_some()
    {
        spans.push(position.start.offset..position.end.offset);
    }
    for child in node.children().into_iter().flatten() {
        markdown_link_spans(child, targets, spans);
    }
}

pub(super) fn web_link_spans(buffer: &Rope, scope: LinkScope) -> Vec<Range<usize>> {
    let (source, start) = match scope {
        LinkScope::Line(line) => (
            buffer.line(line, LineType::LF).to_string(),
            buffer.line_to_byte_idx(line, LineType::LF),
        ),
        LinkScope::Markdown => (buffer.to_string(), 0),
    };
    let mut spans: Vec<_> = plain_links(&source)
        .filter(|(_, address)| parse_plain_url(address).is_some())
        .map(|(span, _)| start + span.start..start + span.end)
        .collect();
    if scope == LinkScope::Markdown
        && let Ok(ast) = markdown::to_mdast(&source, &ParseOptions::gfm())
    {
        let mut targets = HashMap::new();
        definitions(&ast, &mut targets);
        markdown_link_spans(&ast, &targets, &mut spans);
    }
    spans.sort_unstable_by_key(|span| span.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for span in spans {
        if let Some(last) = merged.last_mut()
            && span.start <= last.end
        {
            last.end = last.end.max(span.end);
        } else {
            merged.push(span);
        }
    }
    merged
}
