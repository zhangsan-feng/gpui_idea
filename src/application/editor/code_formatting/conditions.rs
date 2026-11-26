use super::core::{LINE_WIDTH, columns, parser, signature};
use std::{collections::HashMap, ops::Range};
use tree_sitter::Node;

struct Operator {
    row: usize,
    scope: usize,
    gap: Range<usize>,
    column: usize,
    suffix_width: usize,
}

/// Comparison columns are settled first, then trailing logical operators are
/// aligned against the resulting text. No tokens or newlines are moved.
pub(super) fn align_conditions(text: String, language: &str) -> String {
    let text = align(text, language, false);
    align(text, language, true)
}

fn align(text: String, language: &str, logical: bool) -> String {
    let Some(mut parser) = parser(language) else {
        return text;
    };
    let Some(tree) = parser.parse(&text, None) else {
        return text;
    };
    if tree.root_node().has_error() {
        return text;
    }
    let mut operators = Vec::new();
    collect(tree.root_node(), &text, logical, &mut operators);
    operators.sort_by_key(|operator| (operator.row, operator.gap.start));
    let mut counts = HashMap::new();
    for operator in &operators {
        *counts.entry(operator.row).or_insert(0usize) += 1;
    }
    // Expressions containing several comparisons on a single row stay intact.
    operators.retain(|operator| counts[&operator.row] == 1);
    let mut edits = Vec::new();
    let mut start = 0;
    while start < operators.len() {
        let mut end = start + 1;
        while end < operators.len()
            && operators[end].row == operators[end - 1].row + 1
            && operators[end].scope == operators[start].scope
        {
            end += 1;
        }
        let group = &operators[start..end];
        if group.len() > 1 {
            let target = group.iter().map(|operator| operator.column).max().unwrap() + 1;
            if group
                .iter()
                .all(|operator| target + operator.suffix_width <= LINE_WIDTH)
            {
                for operator in group {
                    let padding = " ".repeat(target - operator.column);
                    if text[operator.gap.clone()] != padding {
                        edits.push((operator.gap.clone(), padding));
                    }
                }
            }
        }
        start = end;
    }
    if edits.is_empty() {
        return text;
    }
    let changes = edits.len();
    let mut candidate = text.clone();
    for (range, padding) in edits.into_iter().rev() {
        candidate.replace_range(range, &padding);
    }
    let Some(parsed) = parser.parse(&candidate, None) else {
        return text;
    };
    if parsed.root_node().has_error()
        || signature(tree.root_node(), &text) != signature(parsed.root_node(), &candidate)
    {
        tracing::debug!(%language, logical, "condition alignment skipped: syntax would change");
        return text;
    }
    tracing::info!(%language, logical, changes, "multiline condition operators aligned");
    candidate
}

fn collect(node: Node<'_>, text: &str, logical: bool, operators: &mut Vec<Operator>) {
    let kind = node.kind();
    if kind.contains("string") || kind.contains("comment") || kind.contains("template") {
        return;
    }
    let selected = if logical {
        logical_operator(kind)
    } else {
        matches!(
            kind,
            "==" | "!=" | "<=" | ">=" | "<" | ">" | "===" | "!==" | "<>"
        )
    };
    if selected && !node.is_named() {
        if let Some(operator) = operator(node, text, logical) {
            operators.push(operator);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, text, logical, operators);
    }
}

fn operator(node: Node<'_>, text: &str, logical: bool) -> Option<Operator> {
    let mut expression = node.parent()?;
    if !expression_node(expression.kind()) {
        return None;
    }
    let mut scope = None;
    loop {
        let mut cursor = expression.walk();
        if expression
            .children(&mut cursor)
            .any(|child| logical_operator(child.kind()))
        {
            scope = Some(expression);
        }
        let Some(parent) = expression.parent() else {
            break;
        };
        if !expression_node(parent.kind()) {
            break;
        }
        expression = parent;
    }
    let scope = scope?;
    if scope.start_position().row == scope.end_position().row {
        return None;
    }
    let line_start = text[..node.start_byte()].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[node.end_byte()..]
        .find('\n')
        .map_or(text.len(), |i| node.end_byte() + i);
    if logical && !text[node.end_byte()..line_end].trim().is_empty() {
        return None;
    }
    let prefix = &text[line_start..node.start_byte()];
    let left = prefix.trim_end_matches([' ', '\t']);
    // Preserve leading-operator layouts chosen by the language formatter.
    if left.trim().is_empty() {
        return None;
    }
    Some(Operator {
        row: node.start_position().row,
        scope: scope.id(),
        gap: line_start + left.len()..node.start_byte(),
        column: columns(left),
        suffix_width: columns(&text[node.start_byte()..line_end]),
    })
}

fn expression_node(kind: &str) -> bool {
    matches!(
        kind,
        "binary_expression"
            | "boolean_operator"
            | "comparison_operator"
            | "parenthesized_expression"
    )
}

fn logical_operator(kind: &str) -> bool {
    matches!(kind, "&&" | "||" | "and" | "or")
}
