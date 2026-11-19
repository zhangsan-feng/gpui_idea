use gpui_kit::component::highlighter::LanguageRegistry;
use std::ops::Range;
use tree_sitter::{Node, Parser};
use unicode_width::UnicodeWidthChar;

const TAB_SIZE: usize = 4;
const LINE_WIDTH: usize = 120;

struct Assignment {
    row: usize,
    scope: usize,
    indent: String,
    gap: Range<usize>,
    column: usize,
    line_width: usize,
    operator: String,
}

/// Align syntax-recognized assignments and Go keyed literal elements.
pub(crate) fn align_assignments(text: String, language: &str) -> String {
    let Some(mut parser) = parser(language) else {
        return text;
    };
    let Some(tree) = parser.parse(&text, None) else {
        return text;
    };
    if tree.root_node().has_error() {
        return text;
    }
    let mut assignments = Vec::new();
    collect(tree.root_node(), &text, language == "go", &mut assignments);
    assignments.sort_by_key(|assignment| (assignment.row, assignment.gap.start));
    // Multiple declarations/assignments on one line are not alignment groups.
    let mut counts = std::collections::HashMap::new();
    for assignment in &assignments {
        *counts.entry(assignment.row).or_insert(0usize) += 1;
    }
    assignments.retain(|assignment| counts[&assignment.row] == 1);
    let mut edits = Vec::new();
    let mut start = 0;
    while start < assignments.len() {
        let mut end = start + 1;
        while end < assignments.len() {
            let previous = &assignments[end - 1];
            let next = &assignments[end];
            if next.row != previous.row + 1
                || next.scope != previous.scope
                || next.indent != previous.indent
                || next.operator != previous.operator
            {
                break;
            }
            end += 1;
        }
        let group = &assignments[start..end];
        if group.len() > 1 {
            let target = group
                .iter()
                .map(|assignment| assignment.column)
                .max()
                .unwrap()
                + 1;
            // Keep the long-line pass effective and avoid enormous padding.
            if group.iter().all(|assignment| {
                target + assignment.line_width.saturating_sub(assignment.column) <= LINE_WIDTH
            }) {
                for assignment in group {
                    let mut replacement = " ".repeat(target - assignment.column);
                    if assignment.operator == ":" {
                        replacement.push_str(": ");
                    }
                    if text[assignment.gap.clone()] != replacement {
                        edits.push((assignment.gap.clone(), replacement));
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
    for (range, replacement) in edits.into_iter().rev() {
        candidate.replace_range(range, &replacement);
    }
    let Some(parsed) = parser.parse(&candidate, None) else {
        return text;
    };
    if parsed.root_node().has_error()
        || super::core::signature(tree.root_node(), &text)
            != super::core::signature(parsed.root_node(), &candidate)
    {
        tracing::debug!(%language, "assignment alignment skipped: syntax would change");
        return text;
    }
    tracing::info!(%language, changes, "consecutive assignments and keyed elements aligned");
    candidate
}

fn collect(
    node: Node<'_>,
    text: &str,
    align_keyed_elements: bool,
    assignments: &mut Vec<Assignment>,
) {
    let kind = node.kind();
    if kind.contains("string") || kind.contains("comment") || kind.contains("template") {
        return;
    }
    if (matches!(kind, "=" | ":=") || (align_keyed_elements && kind == ":")) && !node.is_named() {
        if let Some(assignment) = assignment(node, text) {
            assignments.push(assignment);
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, text, align_keyed_elements, assignments);
    }
}

fn assignment(operator: Node<'_>, text: &str) -> Option<Assignment> {
    let mut statement = operator.parent()?;
    let keyed_element = operator.kind() == ":";
    // Excludes default parameters, HTML attributes, comparisons and type syntax.
    let eligible = if keyed_element {
        statement.kind() == "keyed_element"
    } else {
        matches!(
            statement.kind(),
            "let_declaration"
                | "const_item"
                | "static_item"
                | "variable_declarator"
                | "init_declarator"
                | "assignment"
                | "assignment_expression"
                | "assignment_statement"
                | "short_var_declaration"
                | "var_spec"
                | "const_spec"
                | "property_element"
                | "static_variable_declaration"
                | "public_field_definition"
                | "pair"
        )
    };
    if !eligible {
        return None;
    }
    let row = operator.start_position().row;
    if !single_line(statement, text, row) {
        return None;
    }
    // Lift declaration/statement wrappers only. Never cross a block or control
    // flow boundary, including single-line `if` bodies and loop initializers.
    while let Some(parent) = statement.parent() {
        if !matches!(
            parent.kind(),
            "expression_statement"
                | "lexical_declaration"
                | "variable_declaration"
                | "local_variable_declaration"
                | "field_declaration"
                | "property_declaration"
                | "export_statement"
                | "declaration"
                | "const_declaration"
                | "var_declaration"
        ) || !single_line(parent, text, row)
        {
            break;
        }
        statement = parent;
    }
    let line_start = text[..operator.start_byte()]
        .rfind('\n')
        .map_or(0, |i| i + 1);
    let indent = &text[line_start..statement.start_byte()];
    if !indent.chars().all(|ch| matches!(ch, ' ' | '\t')) {
        return None;
    }
    let prefix = &text[line_start..operator.start_byte()];
    let left = prefix.trim_end_matches([' ', '\t']);
    if left.len() <= indent.len() {
        return None;
    }
    let line_end = text[operator.end_byte()..]
        .find('\n')
        .map_or(text.len(), |i| operator.end_byte() + i);
    let suffix = text[operator.end_byte()..line_end].trim_end_matches('\r');
    // A right-hand side on a later line is not a single-line assignment.
    if suffix.trim().is_empty() {
        return None;
    }
    let (gap_end, right_width) = if keyed_element {
        let value = suffix.trim_start_matches([' ', '\t']);
        (
            operator.end_byte() + suffix.len() - value.len(),
            2 + columns(value),
        )
    } else {
        (
            operator.start_byte(),
            columns(&text[operator.start_byte()..line_end]),
        )
    };
    let column = columns(left);
    Some(Assignment {
        row,
        scope: statement
            .parent()
            .map_or(statement.id(), |parent| parent.id()),
        indent: indent.into(),
        gap: line_start + left.len()..gap_end,
        column,
        // Measure without existing padding, so reformatting is idempotent.
        line_width: column + right_width,
        operator: text[operator.byte_range()].into(),
    })
}

pub(super) fn columns(text: &str) -> usize {
    text.chars().fold(0, |column, ch| {
        if ch == '\t' {
            column + TAB_SIZE - column % TAB_SIZE
        } else {
            column + ch.width().unwrap_or(0)
        }
    })
}

pub(super) fn parser(language: &str) -> Option<Parser> {
    let grammar_name = match language {
        "javascriptreact" => "javascript",
        "typescriptreact" => "tsx",
        other => other,
    };
    let grammar = LanguageRegistry::singleton()
        .language(grammar_name)?
        .language?;
    let mut parser = Parser::new();
    parser.set_language(&grammar).ok()?;
    Some(parser)
}

fn single_line(node: Node<'_>, text: &str, row: usize) -> bool {
    node.start_position().row == row
        && !text[node.byte_range()]
            .trim_end_matches(['\r', '\n'])
            .contains('\n')
}
