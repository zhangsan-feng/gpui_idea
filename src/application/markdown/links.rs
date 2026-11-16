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
        _ => node
            .children()?
            .iter()
            .find_map(|child| destination(child, offset, targets)),
    }
}

pub(super) fn web_link_at_offset(source: &str, offset: usize) -> Option<String> {
    if offset > source.len() || !source.is_char_boundary(offset) {
        return None;
    }
    let ast = markdown::to_mdast(source, &ParseOptions::gfm()).ok()?;
    let mut targets = HashMap::new();
    definitions(&ast, &mut targets);
    let target = destination(&ast, offset, &targets)?;
    let url = url::Url::parse(target).ok()?;
    (matches!(url.scheme(), "http" | "https") && url.host_str().is_some()).then(|| url.to_string())
}
