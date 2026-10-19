#[cfg(windows)]
mod script;
#[cfg(windows)]
pub(super) use script::CommandScript;

use super::ShellLaunch;
use alacritty_terminal::tty::{Options, Shell};
use anyhow::Result;
use std::path::Path;

#[cfg(windows)]
pub(super) fn prepare(root: &Path, command: Option<&str>) -> Result<ShellLaunch> {
    let program = std::env::var("COMSPEC")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "cmd.exe".into());
    let mut options = Options {
        working_directory: Some(root.to_owned()),
        drain_on_exit: true,
        ..Default::default()
    };
    let cargo_keys = crate::infrastructure::process_environment::cargo_runtime_keys();
    let script = if command.is_some() || !cargo_keys.is_empty() {
        Some(CommandScript::new(command, &cargo_keys)?)
    } else {
        None
    };
    let startup = if let Some(script) = &script {
        options.env.insert(
            "GPUI_TERMINAL_SCRIPT".into(),
            script.path().to_string_lossy().into_owned(),
        );
        "\"chcp 65001 >nul & \"%GPUI_TERMINAL_SCRIPT%\"\"".into()
    } else {
        "chcp 65001 >nul".into()
    };
    options.shell = Some(Shell::new(
        format!("\"{program}\""),
        vec![
            "/D".into(),
            "/Q".into(),
            "/V:OFF".into(),
            "/S".into(),
            if command.is_some() { "/C" } else { "/K" }.into(),
            startup,
        ],
    ));
    Ok(ShellLaunch {
        options,
        title: "cmd".into(),
        _script: script,
    })
}

#[cfg(unix)]
pub(super) fn prepare(root: &Path, command: Option<&str>) -> Result<ShellLaunch> {
    // Interactive sessions let Alacritty resolve the account's default shell.
    let shell = command.map(|_| {
        let program = std::env::var("SHELL")
            .ok()
            .filter(|shell| !shell.is_empty())
            .unwrap_or_else(|| "/bin/sh".into());
        Shell::new(
            program,
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
    let cargo_keys = crate::infrastructure::process_environment::cargo_runtime_keys();
    // Keep Alacritty's account-shell resolution and macOS login semantics for
    // default interactive sessions; wrap only explicitly configured run shells.
    if !cargo_keys.is_empty() && options.shell.is_some() {
        let shell = options.shell.take().expect("explicit run shell");
        let mut args = Vec::new();
        for key in cargo_keys {
            args.extend(["-u".into(), key]);
        }
        args.push(shell.program);
        args.extend(shell.args);
        options.shell = Some(Shell::new("/usr/bin/env".into(), args));
    }
    Ok(ShellLaunch {
        options,
        title: "Terminal".into(),
    })
}
