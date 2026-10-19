use anyhow::{Context, Result, bail};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

/// Preserve multiline commands, percent expansion and batch exit codes.
pub(in crate::infrastructure::terminal) struct CommandScript {
    directory: PathBuf,
}

impl CommandScript {
    pub(super) fn new(command: Option<&str>, cargo_keys: &[String]) -> Result<Self> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(1);
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        for _ in 0..16 {
            let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "gpui-terminal-{}-{stamp}-{sequence}",
                std::process::id()
            ));
            match std::fs::create_dir(&directory) {
                Ok(()) => {
                    let script = Self { directory };
                    let mut content = String::from("@echo off\r\n");
                    for key in cargo_keys {
                        content.push_str(&format!("set \"{key}=\"\r\n"));
                    }
                    if let Some(command) = command {
                        content.push_str(&command.replace("\r\n", "\n").replace('\n', "\r\n"));
                        content.push_str("\r\nexit /b %errorlevel%\r\n");
                    }
                    std::fs::write(script.path(), content)
                        .context("write terminal command script")?;
                    return Ok(script);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error).context("create terminal command directory"),
            }
        }
        bail!("无法创建终端命令目录")
    }

    pub(super) fn path(&self) -> PathBuf {
        self.directory.join("launch.cmd")
    }
}

impl Drop for CommandScript {
    fn drop(&mut self) {
        // Delete only the file and empty directory created by this instance.
        let _ = std::fs::remove_file(self.path());
        if let Err(error) = std::fs::remove_dir(&self.directory) {
            tracing::warn!(path = %self.directory.display(), %error, "terminal script cleanup failed");
        }
    }
}
