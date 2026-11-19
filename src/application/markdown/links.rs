use markdown::{ParseOptions, mdast::Node};
use std::collections::HashMap;

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
    source: &'a str,
    offset: usize,
    targets: &'a HashMap<String, String>,
) -> Option<&'a str> {
    let position = node.position()?;
    if offset < position.start.offset || offset > position.end.offset {
        return None;
    }
    match node {
        Node::Link(link) => Some(&link.url),
        Node::LinkReference(link) => targets
            .get(&reference_key(&link.identifier))
            .map(String::as_str),
        // Indented code nodes start at their indentation. Fenced code nodes
        // start at the opening fence, even when the fence itself is indented.
        Node::Code(_) if source[position.start.offset..].starts_with([' ', '\t']) => {
            indented_web_address(source, offset)
        }
        _ => node
            .children()?
            .iter()
            .find_map(|child| destination(child, source, offset, targets)),
    }
}

fn indented_web_address(source: &str, offset: usize) -> Option<&str> {
    let line_start = source[..offset].rfind(['\r', '\n']).map_or(0, |i| i + 1);
    let line = source[line_start..].split(['\r', '\n']).next()?;
    let content = line.trim_start_matches([' ', '\t']);
    let address = content.trim_end_matches([' ', '\t']);
    let address_start = line_start + line.len() - content.len();
    let scheme = address.split_once("://")?.0;
    if offset < address_start
        || offset > address_start + address.len()
        || address.contains(char::is_whitespace)
        || !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
    {
        return None;
    }
    Some(address)
}

pub(super) fn web_link_at_offset(source: &str, offset: usize) -> Option<String> {
    if offset > source.len() || !source.is_char_boundary(offset) {
        return None;
    }
    let ast = markdown::to_mdast(source, &ParseOptions::gfm()).ok()?;
    let mut targets = HashMap::new();
    definitions(&ast, &mut targets);
    let target = destination(&ast, source, offset, &targets)?;
    let url = url::Url::parse(target).ok()?;
    (matches!(url.scheme(), "http" | "https") && url.host_str().is_some()).then(|| url.to_string())
}
