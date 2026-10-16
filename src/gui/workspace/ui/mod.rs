pub(super) use crate::data_context::DATA_CONTEXT;
pub(super) mod diagnostics;
mod main_window;

pub(super) use gpui_kit::assets::IconName;
pub(super) use gpui_kit::component::{Icon, Sizable, Size, h_flex, v_flex};
pub(super) use gpui_kit::prelude::FluentBuilder;
pub(super) use gpui_kit::*;

pub(super) use super::{CloseAllDocuments, ProjectFile, SaveDocument, ToolWindow, Workspace};

pub(super) const TOOLBAR: u32 = 0x272a31;
pub(super) const TOOLBAR_HOVER: u32 = 0x3a3e47;
pub(super) const ACTIVITY_BG: u32 = 0xf1f2f4;
pub(super) const PANEL_BG: u32 = 0xf6f7f9;
pub(super) const EDITOR_BG: u32 = 0xffffff;
pub(super) const BORDER: u32 = 0xdfe2e7;
pub(super) const TEXT: u32 = 0x20242b;
pub(super) const MUTED: u32 = 0x737b87;
pub(super) const ACCENT: u32 = 0x3574f0;

pub(super) fn color(value: u32) -> Hsla {
    rgb(value).into()
}

pub(super) fn icon(name: IconName, size: f32, tint: u32) -> Icon {
    Icon::new(name)
        .with_size(Size::Size(px(size)))
        .text_color(color(tint))
}
