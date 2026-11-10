use gpui_kit::{App, Global};
use std::{borrow::Cow, collections::HashSet};

#[derive(Default)]
struct RegisteredFonts(HashSet<String>);
impl Global for RegisteredFonts {}

/// Registers in-memory assets only; file loading belongs to the settings service.
pub(crate) fn register(cx: &mut App) -> anyhow::Result<()> {
    if !cx.has_global::<RegisteredFonts>() {
        cx.set_global(RegisteredFonts::default());
    }
    if !cx.global::<RegisteredFonts>().0.contains("builtin") {
        cx.text_system().add_fonts(vec![
            Cow::Borrowed(include_bytes!("../../../assets/fonts/InterVariable.ttf")),
            Cow::Borrowed(include_bytes!(
                "../../../assets/fonts/JetBrainsMono-Regular.ttf"
            )),
            Cow::Borrowed(include_bytes!(
                "../../../assets/fonts/JetBrainsMono-Bold.ttf"
            )),
            Cow::Borrowed(include_bytes!(
                "../../../assets/fonts/JetBrainsMono-Italic.ttf"
            )),
            Cow::Borrowed(include_bytes!(
                "../../../assets/fonts/JetBrainsMono-BoldItalic.ttf"
            )),
        ])?;
        cx.global_mut::<RegisteredFonts>()
            .0
            .insert("builtin".into());
        tracing::info!(
            jetbrains_mono_available = cx
                .text_system()
                .all_font_names()
                .iter()
                .any(|name| name == "JetBrains Mono"),
            "bundled fonts registered"
        );
    }
    for font in crate::data_context::DATA_CONTEXT
        .settings
        .snapshot()
        .fonts
        .iter()
    {
        if cx.global::<RegisteredFonts>().0.contains(&font.id) {
            continue;
        }
        cx.text_system()
            .add_fonts(vec![Cow::Owned(font.bytes.to_vec())])?;
        cx.global_mut::<RegisteredFonts>().0.insert(font.id.clone());
        tracing::info!(families = ?font.families, "imported fonts registered");
    }
    Ok(())
}
