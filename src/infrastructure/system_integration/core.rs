use crate::domain::settings::{ContextMenuAction, ContextMenuStatus};
use anyhow::Result;

#[cfg(windows)]
pub(super) fn context_menu(action: ContextMenuAction) -> Result<ContextMenuStatus> {
    use super::registry;
    use anyhow::{Context as _, ensure};

    let executable = std::env::current_exe().context("无法获取程序路径")?;
    let executable = executable.to_str().context("程序路径不是有效的 Unicode")?;
    let executable = if let Some(path) = executable.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{path}")
    } else {
        executable
            .strip_prefix(r"\\?\")
            .unwrap_or(executable)
            .to_owned()
    };
    match action {
        ContextMenuAction::Refresh => {}
        ContextMenuAction::Install => registry::install(&executable)?,
        ContextMenuAction::Remove => registry::remove()?,
    }
    let status = registry::status(&executable)?;
    ensure!(
        match action {
            ContextMenuAction::Refresh => true,
            ContextMenuAction::Install => status.installed,
            ContextMenuAction::Remove => !status.has_entries,
        },
        "右键菜单未完整更新，请刷新状态后重试"
    );
    tracing::info!(
        ?action,
        installed = status.installed,
        has_entries = status.has_entries,
        "Windows context menu operation completed"
    );
    Ok(status)
}

#[cfg(not(windows))]
pub(super) fn context_menu(action: ContextMenuAction) -> Result<ContextMenuStatus> {
    anyhow::ensure!(
        matches!(action, ContextMenuAction::Refresh),
        "当前系统暂不支持右键菜单集成"
    );
    Ok(ContextMenuStatus::default())
}
