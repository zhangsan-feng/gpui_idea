use crate::domain::theme::{ImportedTheme, ThemeToken};
use anyhow::{Result, ensure};
use serde_json::Value;

// Approximate TextMate scopes with the categories supported by our Tree-sitter renderer.
const SCOPES: &[(&str, &[&str])] = &[
    (
        "keyword",
        &[
            "keyword.control",
            "storage.type",
            "storage.modifier",
            "keyword",
        ],
    ),
    (
        "string",
        &["string.quoted.double", "string.quoted", "string"],
    ),
    ("string.escape", &["constant.character.escape"]),
    ("string.regex", &["string.regexp"]),
    (
        "comment",
        &["comment.line.double-slash", "comment.line", "comment"],
    ),
    ("comment_doc", &["comment.block.documentation", "comment"]),
    ("number", &["constant.numeric.integer", "constant.numeric"]),
    (
        "boolean",
        &["constant.language.boolean", "constant.language"],
    ),
    ("function", &["entity.name.function", "support.function"]),
    (
        "preproc",
        &["entity.name.function.macro", "meta.preprocessor"],
    ),
    (
        "type",
        &["entity.name.type", "support.type", "support.class"],
    ),
    (
        "constructor",
        &[
            "entity.name.type.class",
            "entity.name.type",
            "support.class",
        ],
    ),
    ("enum", &["entity.name.type.enum", "entity.name.type"]),
    (
        "variable",
        &["variable.other.readwrite", "variable.other", "variable"],
    ),
    ("variable.special", &["variable.language", "variable"]),
    (
        "property",
        &[
            "variable.other.property",
            "support.variable.property",
            "variable.other",
        ],
    ),
    (
        "constant",
        &["variable.other.constant", "constant.other", "constant"],
    ),
    (
        "attribute",
        &[
            "entity.other.attribute-name",
            "entity.name.function.decorator",
        ],
    ),
    ("tag", &["entity.name.tag"]),
    ("tag.doctype", &["meta.tag.sgml.doctype"]),
    ("operator", &["keyword.operator"]),
    ("punctuation", &["punctuation"]),
    (
        "punctuation.bracket",
        &["punctuation.section", "punctuation"],
    ),
    (
        "punctuation.delimiter",
        &["punctuation.separator", "punctuation"],
    ),
    ("title", &["markup.heading"]),
    ("emphasis", &["markup.italic"]),
    ("emphasis.strong", &["markup.bold"]),
    (
        "link_text",
        &["markup.underline.link", "string.other.link.title"],
    ),
    ("link_uri", &["markup.underline.link", "string.other.link"]),
    ("text.code.span", &["markup.inline.raw", "markup.raw"]),
    ("text.literal", &["markup.raw.block", "markup.raw"]),
];

pub(super) fn color(value: &str) -> Option<String> {
    let hex = value.strip_prefix('#')?;
    if !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(if hex.len() <= 4 {
        format!(
            "#{}",
            hex.chars().flat_map(|ch| [ch, ch]).collect::<String>()
        )
    } else {
        value.to_owned()
    })
}

pub(super) fn convert(
    value: &Value,
    source: &str,
    label: Option<&str>,
    mode: Option<&str>,
) -> Result<ImportedTheme> {
    ensure!(
        value["colors"].is_object() || value["tokenColors"].is_array(),
        "文件未包含 VS Code 颜色主题数据"
    );
    let name = label
        .or(value["name"].as_str())
        .unwrap_or(source)
        .to_owned();
    let mode = mode.or(value["type"].as_str());
    let mut theme = ImportedTheme {
        package_name: name.clone(),
        name,
        source: source.into(),
        ..Default::default()
    };
    if let Some(colors) = value["colors"].as_object() {
        for (key, value) in colors {
            if let Some(color) = value.as_str().and_then(color) {
                theme.colors.insert(key.clone(), color);
            }
        }
    }
    if let Some(rules) = value["tokenColors"].as_array() {
        for rule in rules {
            if rule.get("scope").is_none() || rule["scope"].as_str() == Some("") {
                for (property, target) in [
                    ("foreground", "editor.foreground"),
                    ("background", "editor.background"),
                ] {
                    if !theme.colors.contains_key(target) {
                        if let Some(color) = rule["settings"][property].as_str().and_then(color) {
                            theme.colors.insert(target.into(), color);
                        }
                    }
                }
            }
        }
        let mut complex = false;
        for &(capture, candidates) in SCOPES {
            let mut token = ThemeToken::default();
            let mut best = 0;
            for rule in rules {
                let scopes: Vec<&str> = match &rule["scope"] {
                    Value::String(scope) => scope.split(',').map(str::trim).collect(),
                    Value::Array(scopes) => scopes
                        .iter()
                        .filter_map(Value::as_str)
                        .flat_map(|scope| scope.split(',').map(str::trim))
                        .collect(),
                    Value::Null => vec![""],
                    _ => continue,
                };
                for scope in scopes {
                    if scope.contains([' ', '\t', '\n', '>', '&', '|', '(']) {
                        complex = true;
                        continue;
                    }
                    let score = if scope.is_empty() {
                        1
                    } else {
                        if !candidates.iter().any(|candidate| {
                            *candidate == scope
                                || candidate
                                    .strip_prefix(scope)
                                    .is_some_and(|suffix| suffix.starts_with('.'))
                        }) {
                            continue;
                        }
                        scope.len() + 1
                    };
                    if score < best {
                        continue;
                    }
                    best = score;
                    let settings = &rule["settings"];
                    if let Some(color) = settings["foreground"].as_str().and_then(color) {
                        token.foreground = Some(color);
                    }
                    if let Some(style) = settings["fontStyle"].as_str() {
                        token.font_style = Some(style.into());
                    }
                }
            }
            if token != ThemeToken::default() {
                theme.syntax.insert(capture.into(), token);
            }
        }
        if complex {
            theme
                .notes
                .push("包含依赖语言或上下文的 TextMate 规则，已按通用语法类别近似适配。".into());
        }
    }
    theme.dark = match mode {
        Some("light" | "vs" | "hc-light") => false,
        Some("dark" | "vs-dark" | "hc-black") => true,
        _ => theme
            .colors
            .get("editor.background")
            .map(|color| {
                let rgb = u32::from_str_radix(&color[1..7], 16).unwrap_or(0xffffff);
                let brightness =
                    ((rgb >> 16) & 255) * 299 + ((rgb >> 8) & 255) * 587 + (rgb & 255) * 114;
                brightness < 128000
            })
            .unwrap_or(true),
    };
    if value.get("semanticTokenColors").is_some() {
        theme
            .notes
            .push("主题包含语义配色；当前使用语法配色，暂未应用语义规则。".into());
    }
    let mut hash = 0xcbf29ce484222325u64;
    for byte in format!("{}\0{}", source, theme.name).bytes() {
        hash = (hash ^ byte as u64).wrapping_mul(0x100000001b3);
    }
    theme.id = format!("vscode-{hash:016x}");
    Ok(theme)
}
