use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
pub(super) fn validate(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() {
        bail!("请选择项目的完整目录路径");
    }
    let path = path.canonicalize().context("项目目录不存在或无法访问")?;
    if !path.is_dir() {
        bail!("项目必须是目录");
    }
    Ok(path)
}
pub(super) fn launch(path: &Path) -> Result<()> {
    let path = validate(path)?;
    let executable = std::env::current_exe().context("无法定位应用程序")?;
    let mut command = Command::new(executable);
    command.arg("--project").arg(&path).current_dir(&path);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(0x08000000);
    }
    let process = command.spawn().context("无法打开新的项目工作区")?;
    tracing::info!(path = %path.display(), pid = process.id(), "project workspace process launched");
    Ok(())
}
