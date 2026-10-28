mod core;
mod ui;

use super::ProjectManagerWindow;
use crate::domain::project_creation::{ProjectLanguage, ToolchainInfo};
use gpui_kit::component::input::InputState;
use gpui_kit::{AppContext as _, Context, Entity, Window};

pub(super) struct CreateForm {
    pub language: ProjectLanguage,
    pub library: bool,
    pub toolchain: Entity<InputState>,
    pub module: Entity<InputState>,
    pub environment: Entity<InputState>,
    pub info: Option<ToolchainInfo>,
    pub probing: bool,
    pub generation: u64,
}

impl CreateForm {
    pub(super) fn new(window: &mut Window, cx: &mut Context<ProjectManagerWindow>) -> Self {
        Self {
            language: ProjectLanguage::Rust,
            library: false,
            toolchain: cx
                .new(|cx| InputState::new(window, cx).placeholder("自动检测或填写可执行文件路径")),
            module: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("留空使用项目目录名，例如 example.com/my-project")
            }),
            environment: cx.new(|cx| {
                InputState::new(window, cx).placeholder("KEY=value; KEY2=value（用于初始化命令）")
            }),
            info: None,
            probing: false,
            generation: 0,
        }
    }
}
