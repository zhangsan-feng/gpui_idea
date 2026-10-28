pub(super) use crate::data_context::DATA_CONTEXT;
pub(super) mod diagnostics;
mod main_window;

pub(super) use gpui_kit::assets::IconName;
pub(super) use gpui_kit::component::{Icon, Sizable, h_flex, v_flex};
pub(super) use gpui_kit::prelude::FluentBuilder;
pub(super) use gpui_kit::*;

pub(super) use super::{CloseAllDocuments, ProjectFile, SaveDocument, ToolWindow, Workspace};

pub(super) use crate::gui::component::{icon::icon, tokens::*};
