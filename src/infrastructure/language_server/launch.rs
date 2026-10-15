use anyhow::{Context, Result, bail};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, Command};

pub(super) fn family(language: &str) -> Result<&'static str> {
    match language {
        "rust" => Ok("rust"),
        _ => bail!("当前仅支持 Rust 语言服务"),
    }
}

fn executable() -> Option<PathBuf> {
    for directory in std::env::split_paths(&std::env::var_os("PATH")?) {
        let candidate = directory.join(if cfg!(windows) {
            "rust-analyzer.exe"
        } else {
            "rust-analyzer"
        });
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn spawn(root: &Path, language: &str) -> Result<Child> {
    family(language)?;
    let path = executable().context("无法启动 Rust 语言服务：PATH 中未找到 rust-analyzer")?;
    let mut command = Command::new(path);
    command
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let child = command.spawn().context("无法启动 rust-analyzer")?;
    tracing::info!(root = %root.display(), "Starting rust-analyzer");
    Ok(child)
}
