use gpui_kit::component::highlighter::LanguageRegistry;
use std::ops::Range;
use tree_sitter::{Node, Parser};

const LINE_WIDTH: usize = 120;

/// A conservative shared pass after the language formatter. Only recognized
/// syntax containers are split; strings, comments and arbitrary text stay intact.
pub(crate) fn wrap_long_lines(mut text: String, language: &str) -> String {
    let grammar_name = match language {
        "javascriptreact" => "javascript",
        "typescriptreact" => "tsx",
        other => other,
    };
    let Some(grammar) = LanguageRegistry::singleton()
        .language(grammar_name)
        .and_then(|config| config.language)
    else {
        return text;
    };
    let mut parser = Parser::new();
    if parser.set_language(&grammar).is_err() {
        return text;
    }
    let Some(mut tree) = parser.parse(&text, None) else {
        return text;
    };
    if tree.root_node().has_error() {
        return text;
    }
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut changes = 0;
    // Bound work for unusually large files. Each successful pass splits one
    // container, allowing nested containers to use their new indentation.
    for _ in 0..256 {
        let mut edits = Vec::new();
        find_wrap(tree.root_node(), &text, language, newline, &mut edits);
        if edits.is_empty() {
            break;
        }
        let mut candidate = text.clone();
        edits.sort_by_key(|(range, _)| range.start);
        for (range, replacement) in edits.into_iter().rev() {
            candidate.replace_range(range, &replacement);
        }
        let Some(parsed) = parser.parse(&candidate, None) else {
            break;
        };
        // Newlines can change meaning (automatic semicolons, Python blocks,
        // etc.). Reject a pass if the named syntax structure or tokens change.
        if parsed.root_node().has_error()
            || signature(tree.root_node(), &text) != signature(parsed.root_node(), &candidate)
        {
            tracing::debug!(%language, "long-line wrapping skipped: syntax would change");
            break;
        }
        text = candidate;
        tree = parsed;
        changes += 1;
    }
    if changes > 0 {
        tracing::info!(%language, changes, width = LINE_WIDTH, "long code lines wrapped");
    }
    text
}

type Edit = (Range<usize>, String);

fn find_wrap(node: Node<'_>, text: &str, language: &str, newline: &str, edits: &mut Vec<Edit>) {
    // Literal and comment contents must never be reformatted by this pass.
    let kind = node.kind();
    if kind.contains("string") || kind.contains("comment") || kind.contains("template") {
        return;
    }
    if node.start_position().row == node.end_position().row {
        let start = text[..node.start_byte()].rfind('\n').map_or(0, |i| i + 1);
        let end = text[node.end_byte()..]
            .find('\n')
            .map_or(text.len(), |i| node.end_byte() + i);
        let line = &text[start..end];
        if line
            .chars()
            .map(|ch| if ch == '\t' { 4 } else { 1 })
            .sum::<usize>()
            > LINE_WIDTH
        {
            let indent: String = line
                .chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .collect();
            let unit = if indent.contains('\t') || language == "go" {
                "\t"
            } else {
                "    "
            };
            let continuation = format!("{newline}{indent}{unit}");
            if wrap_logical_expression(node, text, language, &continuation, edits) {
                return;
            }
            if wrap_chain(node, language, &continuation, edits) {
                return;
            }
            if wrap_list(
                node,
                text,
                language,
                &continuation,
                &format!("{newline}{indent}"),
                edits,
            ) {
                return;
            }
        }
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        find_wrap(child, text, language, newline, edits);
        if !edits.is_empty() {
            return;
        }
    }
}

fn wrap_logical_expression(
    node: Node<'_>,
    text: &str,
    language: &str,
    continuation: &str,
    edits: &mut Vec<Edit>,
) -> bool {
    if !matches!(node.kind(), "binary_expression" | "boolean_operator") {
        return false;
    }
    if language == "python" {
        // Python only allows implicit continuation inside brackets. Keep the
        // simple pass whitespace-only instead of inserting backslashes.
        let mut parent = node.parent();
        let mut enclosed = false;
        while let Some(ancestor) = parent {
            if matches!(
                ancestor.kind(),
                "parenthesized_expression"
                    | "argument_list"
                    | "list"
                    | "dictionary"
                    | "set"
                    | "tuple"
            ) {
                enclosed = true;
                break;
            }
            if ancestor.kind().ends_with("statement") || ancestor.kind() == "lambda" {
                break;
            }
            parent = ancestor.parent();
        }
        if !enclosed {
            return false;
        }
    }
    logical_breaks(node, text, continuation, edits);
    !edits.is_empty()
}

fn logical_breaks(node: Node<'_>, text: &str, continuation: &str, edits: &mut Vec<Edit>) {
    if !matches!(node.kind(), "binary_expression" | "boolean_operator") {
        return;
    }
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    for (index, child) in children.iter().enumerate() {
        if !child.is_named() && matches!(&text[child.byte_range()], "&&" | "||" | "and" | "or") {
            if let Some(next) = children.get(index + 1) {
                let gap = child.end_byte()..next.start_byte();
                // Never replace comments or other syntax between operands.
                if text[gap.clone()].chars().all(char::is_whitespace) {
                    // Breaking after the operator also preserves Go's automatic
                    // semicolon rules and the expression's original precedence.
                    edits.push((gap, continuation.into()));
                }
            }
        } else {
            logical_breaks(*child, text, continuation, edits);
        }
    }
}

fn wrap_list(
    node: Node<'_>,
    text: &str,
    language: &str,
    continuation: &str,
    closing: &str,
    edits: &mut Vec<Edit>,
) -> bool {
    if !matches!(
        node.kind(),
        "argument_list"
            | "arguments"
            | "literal_value"
            | "field_initializer_list"
            | "initializer_list"
            | "object"
            | "object_pattern"
            | "array"
            | "array_expression"
            | "list"
            | "dictionary"
            | "set"
            | "tuple"
    ) {
        return false;
    }
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let (Some(first), Some(last)) = (children.first(), children.last()) else {
        return false;
    };
    if !matches!(
        (&text[first.byte_range()], &text[last.byte_range()]),
        ("{", "}") | ("[", "]") | ("(", ")")
    ) {
        return false;
    }
    let items: Vec<_> = children.iter().filter(|child| child.is_named()).collect();
    if items.is_empty()
        || children
            .iter()
            .any(|child| child.kind().contains("comment"))
    {
        return false;
    }
    edits.push((first.end_byte()..items[0].start_byte(), continuation.into()));
    for child in &children {
        if child.kind() == "," {
            let next = children
                .iter()
                .find(|next| next.start_byte() >= child.end_byte());
            if let Some(next) = next.filter(|next| next.start_byte() < last.start_byte()) {
                edits.push((child.end_byte()..next.start_byte(), continuation.into()));
            }
        }
    }
    let previous = children[children.len() - 2];
    // Go requires a trailing comma before a closing delimiter on a new line.
    let comma = if language == "go" && previous.kind() != "," {
        ","
    } else {
        ""
    };
    edits.push((
        previous.end_byte()..last.start_byte(),
        format!("{comma}{closing}"),
    ));
    true
}

fn wrap_chain(node: Node<'_>, language: &str, continuation: &str, edits: &mut Vec<Edit>) -> bool {
    if !matches!(
        node.kind(),
        "call_expression"
            | "method_invocation"
            | "member_call_expression"
            | "nullsafe_member_call_expression"
    ) {
        return false;
    }
    // Python permits implicit continuation only inside enclosing brackets.
    if language == "python" {
        return false;
    }
    chain_breaks(node, language, continuation, edits);
    !edits.is_empty()
}

fn chain_breaks(node: Node<'_>, language: &str, continuation: &str, edits: &mut Vec<Edit>) -> bool {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let Some(first) = children.first() else {
        return false;
    };
    let receiver_has_call = chain_breaks(*first, language, continuation, edits);
    if receiver_has_call {
        if let Some(operator) = children
            .iter()
            .find(|child| matches!(child.kind(), "." | "?." | "->" | "?->"))
        {
            if language == "go" {
                if let Some(next) = children
                    .iter()
                    .find(|child| child.start_byte() >= operator.end_byte())
                {
                    edits.push((operator.end_byte()..next.start_byte(), continuation.into()));
                }
            } else {
                edits.push((first.end_byte()..operator.start_byte(), continuation.into()));
            }
        }
    }
    // Only follow the receiver/function, never expressions inside arguments.
    receiver_has_call
        || matches!(
            node.kind(),
            "call_expression"
                | "method_invocation"
                | "member_call_expression"
                | "nullsafe_member_call_expression"
        )
}

fn signature(root: Node<'_>, text: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        if node.is_named() {
            result.push((
                node.kind().into(),
                if node.child_count() == 0 {
                    text[node.byte_range()].into()
                } else {
                    String::new()
                },
            ));
        } else if node.child_count() == 0 && node.kind() != "," {
            result.push((node.kind().into(), text[node.byte_range()].into()));
        }
        if cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return result;
            }
            result.push((")".into(), String::new()));
        }
    }
}
