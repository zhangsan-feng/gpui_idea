use crate::data_context::DATA_CONTEXT;
mod bottom;
mod editor;
mod main_window;
mod sidebar;
mod status_bar;

use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Sizable, Size, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use super::{
    CloseAllDocuments, DraggedEditorTab, DraggedProjectEntry, ProjectFile, SaveDocument,
    ToolWindow, Workspace,
};

const TOOLBAR: u32 = 0x272a31;
const TOOLBAR_HOVER: u32 = 0x3a3e47;
const ACTIVITY_BG: u32 = 0xf1f2f4;
const PANEL_BG: u32 = 0xf6f7f9;
const EDITOR_BG: u32 = 0xffffff;
const BORDER: u32 = 0xdfe2e7;
const TEXT: u32 = 0x20242b;
const MUTED: u32 = 0x737b87;
const ACCENT: u32 = 0x3574f0;

fn color(value: u32) -> Hsla {
    rgb(value).into()
}

fn icon(name: IconName, size: f32, tint: u32) -> Icon {
    Icon::new(name)
        .with_size(Size::Size(px(size)))
        .text_color(color(tint))
}
