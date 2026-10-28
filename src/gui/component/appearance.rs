use crate::{data_context::DATA_CONTEXT, domain::settings::ColorTheme};
use gpui_kit::component::{Theme, ThemeMode, highlighter::HighlightTheme};
use gpui_kit::{App, Hsla};

pub(crate) fn apply(cx: &mut App) {
    let appearance = DATA_CONTEXT.settings.get().appearance;
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
        theme.font_family = appearance.ui_font.into();
        theme.mono_font_family = appearance.editor_font.into();
        theme.mono_font_size = gpui_kit::px(appearance.editor_size as f32);
    });
    cx.refresh_windows();
}

pub(crate) fn ui_font() -> gpui_kit::SharedString {
    DATA_CONTEXT.settings.get().appearance.ui_font.into()
}

pub(crate) fn editor_font() -> gpui_kit::SharedString {
    DATA_CONTEXT.settings.get().appearance.editor_font.into()
}

pub(crate) fn color(value: u32) -> Hsla {
    foreground(value)
}

pub(crate) fn foreground(value: u32) -> Hsla {
    let mut color: Hsla = gpui_kit::rgb(value).into();
    if DATA_CONTEXT.settings.get().appearance.theme != ColorTheme::Dark {
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
    if DATA_CONTEXT.settings.get().appearance.theme != ColorTheme::Dark {
        return color;
    }
    match value {
        0x272a31 | 0x3a3e47 => return gpui_kit::rgb(0x202228).into(),
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
    if DATA_CONTEXT.settings.get().appearance.theme == ColorTheme::Dark && color.s < 0.25 {
        gpui_kit::rgb(0x3b3e46).into()
    } else {
        foreground(value)
    }
}
