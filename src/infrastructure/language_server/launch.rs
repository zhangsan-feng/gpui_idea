use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, Command};

use super::languages;

pub(super) fn family(language: &str) -> Result<&'static str> {
    crate::domain::language_server::family(language)
        .with_context(|| format!("尚未接入 {language} 语言服务"))
}

pub(super) fn executable(name: &str) -> Option<PathBuf> {
    for directory in std::env::split_paths(&std::env::var_os("PATH")?) {
        let candidate = directory.join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_owned()
        });
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn spawn(
    root: &Path,
    language: &str,
    preparation: &super::managed::InstallContext,
) -> Result<(Child, Option<String>, Option<PathBuf>)> {
    let family = family(language)?;
    let adapter = languages::get(family)?;
    let (mut command, installed_version) = (adapter.prepare)(root, preparation)?;
    let version_program = adapter
        .native_version
        .map(|_| PathBuf::from(command.as_std().get_program()));
    crate::infrastructure::process_environment::sanitize(command.as_std_mut());
    preparation.before_launch()?;
    preparation.launcher(
        PathBuf::from(command.as_std().get_program()),
        installed_version.clone(),
    );
    command
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let child = command
        .spawn()
        .with_context(|| format!("无法启动 {family} 语言服务"))?;
    tracing::debug!(root = %root.display(), language = family, "Starting language server");
    Ok((child, installed_version, version_program))
}

pub(super) async fn native_version(program: PathBuf, argument: &str) -> Option<String> {
    let mut command = Command::new(program);
    command
        .arg(argument)
        .stdin(Stdio::null())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let output = tokio::time::timeout(std::time::Duration::from_secs(3), command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find(|line| !line.trim().is_empty())
        .and_then(super::inventory::version_label)
}
