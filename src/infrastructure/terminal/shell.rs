use super::ShellLaunch;
use alacritty_terminal::tty::{Options, Shell};
use anyhow::Result;
use std::path::Path;

impl ShellLaunch {
    pub(super) fn release(&self) -> Result<()> {
        #[cfg(windows)]
        std::fs::write(self.workspace.path().join("ready"), [])?;
        Ok(())
    }
}

#[cfg(windows)]
pub(super) fn prepare(root: &Path, command: Option<&str>) -> Result<ShellLaunch> {
    use anyhow::Context;

    let program = std::env::var("COMSPEC")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "cmd.exe".into());
    let mut options = Options {
        working_directory: Some(root.to_owned()),
        drain_on_exit: true,
        ..Default::default()
    };
    let workspace = tempfile::Builder::new()
        .prefix("gpui-terminal-")
        .tempdir()
        .context("create terminal command directory")?;
    let mut script = String::from(
        "@echo off\r\n:gpui_wait\r\nif not exist \"%GPUI_TERMINAL_READY%\" goto gpui_wait\r\n",
    );
    if let Some(command) = command {
        for line in command.lines() {
            script.push_str("echo(\x1b[34m");
            for ch in line.chars() {
                if ch == '%' {
                    script.push('%');
                } else if matches!(ch, '^' | '"' | '&' | '|' | '<' | '>' | '(' | ')') {
                    script.push('^');
                }
                script.push(ch);
            }
            script.push_str("\x1b[0m\r\n");
        }
        script.push_str(&command.replace("\r\n", "\n").replace('\n', "\r\n"));
        script.push_str("\r\nexit /b %errorlevel%\r\n");
    }
    let path = workspace.path().join("launch.cmd");
    std::fs::write(&path, script)?;
    options.env.insert(
        "GPUI_TERMINAL_SCRIPT".into(),
        path.to_string_lossy().into_owned(),
    );
    options.env.insert(
        "GPUI_TERMINAL_READY".into(),
        workspace
            .path()
            .join("ready")
            .to_string_lossy()
            .into_owned(),
    );
    // Switch codepage before reading the UTF-8 script. Its gate uses only cmd
    // builtins, so no descendant can escape the job and no user input is consumed.
    let args = vec![
        "/D".into(),
        "/Q".into(),
        "/V:OFF".into(),
        "/S".into(),
        if command.is_some() { "/C" } else { "/K" }.into(),
        "\"chcp 65001 >nul & \"%GPUI_TERMINAL_SCRIPT%\"\"".into(),
    ];
    options.shell = Some(Shell::new(format!("\"{program}\""), args));
    Ok(ShellLaunch {
        options,
        title: "cmd".into(),
        workspace,
    })
}

#[cfg(unix)]
pub(super) fn prepare(root: &Path, command: Option<&str>) -> Result<ShellLaunch> {
    // Interactive sessions retain Alacritty's $SHELL/passwd and macOS login
    // handling. Runs load the user's login environment first, then execute POSIX
    // syntax through sh, including when the interactive shell is fish.
    let shell = command.map(|_| {
        Shell::new(
            login_shell(),
            vec![
                "-l".into(),
                "-c".into(),
                "exec /bin/sh -c \"$GPUI_TERMINAL_COMMAND\"".into(),
            ],
        )
    });
    let mut options = Options {
        shell,
        working_directory: Some(root.to_owned()),
        drain_on_exit: true,
        ..Default::default()
    };
    options.env.insert("TERM".into(), "xterm-256color".into());
    options.env.insert("COLORTERM".into(), "truecolor".into());
    if let Some(command) = command {
        options
            .env
            .insert("GPUI_TERMINAL_COMMAND".into(), command.into());
    }
    Ok(ShellLaunch {
        options,
        title: "Terminal".into(),
    })
}

#[cfg(unix)]
fn login_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() {
            return shell;
        }
    }
    // GUI apps launched from Finder need not inherit SHELL. Resolve the account's
    // configured shell without spawning another process or assuming zsh/bash.
    let mut buffer = vec![0_u8; 4096];
    loop {
        let mut entry = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let status = unsafe {
            libc::getpwuid_r(
                libc::getuid(),
                entry.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && buffer.len() < 1_048_576 {
            buffer.resize(buffer.len() * 2, 0);
            continue;
        }
        if status == 0 && !result.is_null() {
            let shell = unsafe { (*result).pw_shell };
            if !shell.is_null() {
                let shell = unsafe { std::ffi::CStr::from_ptr(shell) }.to_string_lossy();
                if !shell.is_empty() {
                    return shell.into_owned();
                }
            }
        }
        return "/bin/sh".into();
    }
}
