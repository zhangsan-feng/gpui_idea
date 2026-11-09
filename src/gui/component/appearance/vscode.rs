use crate::domain::theme::ImportedTheme;
use gpui_kit::{
    Hsla,
    component::{Theme, ThemeMode, highlighter::HighlightTheme},
};
use serde_json::{Value, json};

pub(super) fn parse(value: &str) -> Option<Hsla> {
    let hex = value.strip_prefix('#')?;
    let rgba = match hex.len() {
        6 => (u32::from_str_radix(hex, 16).ok()? << 8) | 255,
        8 => u32::from_str_radix(hex, 16).ok()?,
        _ => return None,
    };
    Some(gpui_kit::rgba(rgba).into())
}

pub(super) fn lookup(theme: &ImportedTheme, key: &str) -> Option<Hsla> {
    theme.colors.get(key).and_then(|value| parse(value))
}

pub(super) fn apply(theme: &ImportedTheme, current: &mut Theme) {
    macro_rules! map {
        ($($field:ident => $key:literal),* $(,)?) => { $(if let Some(color) = lookup(theme, $key) { current.colors.$field = color; })* };
    }
    map! {
        background => "editor.background", foreground => "foreground", border => "panel.border",
        muted_foreground => "descriptionForeground", muted => "input.background",
        primary => "button.background", primary_foreground => "button.foreground", primary_hover => "button.hoverBackground",
        button_primary => "button.background", button_primary_foreground => "button.foreground", button_primary_hover => "button.hoverBackground",
        button => "button.secondaryBackground", button_foreground => "button.secondaryForeground", button_hover => "button.secondaryHoverBackground",
        secondary => "button.secondaryBackground", secondary_foreground => "button.secondaryForeground", secondary_hover => "button.secondaryHoverBackground",
        input => "input.border", ring => "focusBorder", caret => "editorCursor.foreground", selection => "editor.selectionBackground",
        list => "sideBar.background", list_hover => "list.hoverBackground", list_active => "list.activeSelectionBackground",
        list_active_border => "list.focusOutline", accent => "list.hoverBackground", accent_foreground => "list.hoverForeground",
        sidebar => "sideBar.background", sidebar_foreground => "sideBar.foreground", sidebar_border => "sideBar.border",
        sidebar_accent => "list.activeSelectionBackground", sidebar_accent_foreground => "list.activeSelectionForeground",
        popover => "menu.background", popover_foreground => "menu.foreground",
        tab => "tab.inactiveBackground", tab_foreground => "tab.inactiveForeground", tab_bar => "editorGroupHeader.tabsBackground",
        tab_active => "tab.activeBackground", tab_active_foreground => "tab.activeForeground",
        title_bar => "titleBar.activeBackground", title_bar_border => "titleBar.border",
        status_bar => "statusBar.background", status_bar_border => "statusBar.border",
        scrollbar => "editor.background", scrollbar_thumb => "scrollbarSlider.background", scrollbar_thumb_hover => "scrollbarSlider.hoverBackground",
        link => "textLink.foreground", link_hover => "textLink.activeForeground",
    }
    let baseline = if theme.dark {
        HighlightTheme::default_dark()
    } else {
        HighlightTheme::default_light()
    };
    let mut value = serde_json::to_value(baseline.as_ref()).unwrap_or(Value::Null);
    value["name"] = json!(theme.name);
    value["appearance"] = json!(if theme.dark { "dark" } else { "light" });
    for (field, key) in [
        ("editor.background", "editor.background"),
        ("editor.foreground", "editor.foreground"),
        (
            "editor.active_line.background",
            "editor.lineHighlightBackground",
        ),
        ("editor.line_number", "editorLineNumber.foreground"),
        (
            "editor.active_line_number",
            "editorLineNumber.activeForeground",
        ),
        ("editor.gutter.background", "editorGutter.background"),
        ("editor.invisible", "editorWhitespace.foreground"),
    ] {
        if let Some(color) = theme.colors.get(key) {
            value["style"][field] = json!(color);
        }
    }
    // Unspecified syntax colors follow the imported foreground, not an unrelated built-in palette.
    if let Some(foreground) = theme.colors.get("editor.foreground") {
        if let Some(syntax) = value["style"]["syntax"].as_object_mut() {
            for token in syntax.values_mut() {
                *token = json!({"color": foreground});
            }
        }
    }
    for (name, token) in &theme.syntax {
        let style = &mut value["style"]["syntax"][name];
        if !style.is_object() {
            *style = json!({});
        }
        if let Some(color) = &token.foreground {
            style["color"] = json!(color);
        }
        if let Some(font) = &token.font_style {
            style["font_style"] = json!(if font.split_whitespace().any(|part| part == "italic") {
                "italic"
            } else {
                "normal"
            });
            style["font_weight"] = json!(if font.split_whitespace().any(|part| part == "bold") {
                700
            } else {
                400
            });
        }
    }
    match serde_json::from_value::<HighlightTheme>(value) {
        Ok(highlight) => current.highlight_theme = std::sync::Arc::new(highlight),
        Err(error) => {
            tracing::error!(%error, name = %theme.name, "failed to apply imported syntax palette")
        }
    }
    current.mode = if theme.dark {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
}
