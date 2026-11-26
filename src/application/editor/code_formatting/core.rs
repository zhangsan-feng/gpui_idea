use super::{alignment, conditions, wrapping};
use gpui_kit::component::highlighter::LanguageRegistry;
use tree_sitter::{Node, Parser};
use unicode_width::UnicodeWidthChar;

pub(super) const LINE_WIDTH: usize = 120;
const TAB_SIZE: usize = 4;

type FormattingStage = fn(String, &str) -> String;

const STAGES: &[FormattingStage] = &[
    wrapping::wrap_long_lines,
    alignment::align_assignments,
    conditions::align_conditions,
];

pub(crate) fn format_code(text: String, language: &str) -> String {
    STAGES
        .iter()
        .fold(text, |source, stage| stage(source, language))
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

pub(super) fn signature(root: Node<'_>, text: &str) -> Vec<(String, String)> {
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
