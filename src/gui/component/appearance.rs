use crate::{data_context::DATA_CONTEXT, domain::settings::ColorTheme};
use gpui_kit::component::{Theme, ThemeMode, highlighter::HighlightTheme};
use gpui_kit::{App, Hsla};
mod vscode;

pub(crate) fn apply(cx: &mut App) {
    let appearance = DATA_CONTEXT.settings.snapshot().general.appearance.clone();
    let dark = appearance.theme == ColorTheme::Dark;
    Theme::change(
        if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        },
        None,
        cx,
    );
    if !dark {
        if let Ok(theme) = serde_json::from_slice::<HighlightTheme>(include_bytes!(
            "../../../assets/editor/idea-light.json"
        )) {
            Theme::update(cx, |current| {
                current.highlight_theme = std::sync::Arc::new(theme)
            });
        }
    }
    Theme::update(cx, |theme| {
        if let Some(imported) = appearance.selected_theme.as_ref().and_then(|id| {
            appearance
                .imported_themes
                .iter()
                .find(|theme| &theme.id == id)
        }) {
            vscode::apply(imported, theme);
            tracing::info!(name = %imported.name, "VS Code theme applied");
        }
        theme.font_family = appearance.ui_font.into();
        theme.mono_font_family = appearance.editor_font.into();
        theme.mono_font_size = gpui_kit::px(appearance.editor_size as f32);
    });
    cx.refresh_windows();
}

pub(crate) fn ui_font() -> gpui_kit::SharedString {
    DATA_CONTEXT
        .settings
        .snapshot()
        .general
        .appearance
        .ui_font
        .clone()
        .into()
}

pub(crate) fn editor_font() -> gpui_kit::SharedString {
    DATA_CONTEXT
        .settings
        .snapshot()
        .general
        .appearance
        .editor_font
        .clone()
        .into()
}

pub(crate) fn color(value: u32) -> Hsla {
    foreground(value)
}

pub(crate) fn foreground(value: u32) -> Hsla {
    let mut color: Hsla = gpui_kit::rgb(value).into();
    let key = if matches!(value, 0xdfe1e5 | 0xd4d7dd | 0xb9bdc5) {
        "titleBar.activeForeground"
    } else if color.s < 0.25 {
        if color.l > 0.9 {
            "button.foreground"
        } else if color.l < 0.3 {
            "foreground"
        } else {
            "descriptionForeground"
        }
    } else {
        match value {
            0x3574f0 | 0x2563eb | 0x1d4ed8 => "textLink.foreground",
            0xb42318 | 0xdc2626 | 0xd44352 => "errorForeground",
            _ => "",
        }
    };
    if let Some(color) = imported_color(key) {
        return color;
    }
    if DATA_CONTEXT.settings.snapshot().general.appearance.theme != ColorTheme::Dark {
        return color;
    }
    if color.l < 0.5 {
        color.l = if color.s > 0.25 {
            0.7
        } else {
            0.88 - color.l * 0.3
        };
    }
    color
}

pub(crate) fn background(value: u32) -> Hsla {
    let mut color: Hsla = gpui_kit::rgb(value).into();
    let key = match value {
        0x272a31 | 0x27282e => "titleBar.activeBackground",
        0x3a3e47 => "toolbar.hoverBackground",
        0xffffff => "editor.background",
        0xf7f8fa | 0xf8fafc | 0xf1f2f4 => "sideBar.background",
        0xe8efff | 0xd5e2ff => "list.activeSelectionBackground",
        0xeef2f7 | 0xe8eaf0 | 0xf7f7fa => "list.hoverBackground",
        _ if color.s < 0.15 && color.l > 0.9 => "editor.background",
        _ => "",
    };
    if let Some(color) = imported_color(key) {
        return color;
    }
    if DATA_CONTEXT.settings.snapshot().general.appearance.theme != ColorTheme::Dark {
        return color;
    }
    match value {
        0x272a31 | 0x27282e | 0x3a3e47 => return gpui_kit::rgb(0x202228).into(),
        0xffffff => return gpui_kit::rgb(0x1e1f22).into(),
        _ => {}
    }
    if color.l > 0.85 {
        color.l = if color.s > 0.25 {
            0.25
        } else {
            0.16 + (1.0 - color.l) * 0.3
        };
    }
    color
}

pub(crate) fn border(value: u32) -> Hsla {
    let color: Hsla = gpui_kit::rgb(value).into();
    if color.s < 0.25 {
        if let Some(color) =
            imported_color("panel.border").or_else(|| imported_color("contrastBorder"))
        {
            return color;
        }
    }
    if DATA_CONTEXT.settings.snapshot().general.appearance.theme == ColorTheme::Dark
        && color.s < 0.25
    {
        gpui_kit::rgb(0x3b3e46).into()
    } else {
        foreground(value)
    }
}

fn imported_color(key: &str) -> Option<Hsla> {
    let snapshot = DATA_CONTEXT.settings.snapshot();
    let appearance = &snapshot.general.appearance;
    let id = appearance.selected_theme.as_ref()?;
    let theme = appearance
        .imported_themes
        .iter()
        .find(|theme| &theme.id == id)?;
    vscode::lookup(theme, key).or_else(|| {
        if key == "foreground" {
            vscode::lookup(theme, "editor.foreground")
        } else {
            None
        }
    })
}

pub(crate) fn preview_color(
    appearance: &crate::domain::settings::AppearanceSettings,
    key: &str,
    fallback: Hsla,
) -> Hsla {
    appearance
        .selected_theme
        .as_ref()
        .and_then(|id| {
            appearance
                .imported_themes
                .iter()
                .find(|theme| &theme.id == id)
        })
        .and_then(|theme| vscode::lookup(theme, key))
        .unwrap_or(fallback)
}

/// Read a semantic VS Code color, retaining the existing palette as fallback.
pub(crate) fn themed(key: &str, fallback: Hsla) -> Hsla {
    imported_color(key).unwrap_or(fallback)
}
