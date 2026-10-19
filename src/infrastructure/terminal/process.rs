use alacritty_terminal::tty::Pty;
use anyhow::{Context, Result, ensure};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) fn id(pty: &Pty) -> Result<u32> {
    #[cfg(windows)]
    let id = pty
        .child_watcher()
        .pid()
        .context("terminal shell has no process id")?
        .get();
    #[cfg(unix)]
    let id = pty.child().id();
    ensure!(id > 1, "invalid terminal process id");
    Ok(id)
}

#[cfg(windows)]
pub(super) fn terminate(id: u32, exited: &AtomicBool) -> Result<()> {
    use std::os::windows::process::CommandExt;
    if exited.load(Ordering::Acquire) {
        return Ok(());
    }
    let taskkill = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .context("SystemRoot is unavailable")?
        .join("System32/taskkill.exe");
    let output = std::process::Command::new(taskkill)
        .args(["/PID", &id.to_string(), "/T", "/F"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW, using the standard library.
        .output()
        .context("stop terminal process tree")?;
    ensure!(
        output.status.success() || exited.load(Ordering::Acquire),
        "terminal process tree stop failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}

#[cfg(unix)]
pub(super) fn terminate(id: u32, exited: &AtomicBool) -> Result<()> {
    if exited.load(Ordering::Acquire) {
        return Ok(());
    }
    let output = std::process::Command::new("/bin/ps")
        .args(["-axo", "pid=,ppid=,pgid="])
        .output()
        .context("enumerate terminal processes")?;
    ensure!(
        output.status.success(),
        "cannot enumerate terminal processes"
    );
    let rows: Vec<(u32, u32, u32)> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut values = line.split_whitespace().map(str::parse::<u32>);
            Some((
                values.next()?.ok()?,
                values.next()?.ok()?,
                values.next()?.ok()?,
            ))
        })
        .collect();
    let mut descendants = std::collections::HashSet::from([id]);
    loop {
        let before = descendants.len();
        for (pid, parent, _) in &rows {
            if descendants.contains(parent) {
                descendants.insert(*pid);
            }
        }
        if descendants.len() == before {
            break;
        }
    }
    // Alacritty starts a new session; only signal groups led by this process tree.
    let groups: std::collections::BTreeSet<_> = rows
        .iter()
        .filter(|(pid, _, group)| descendants.contains(pid) && descendants.contains(group))
        .map(|(_, _, group)| *group)
        .collect();
    if exited.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut command = std::process::Command::new("/bin/kill");
    command.arg("-KILL");
    for group in groups {
        if group > 1 {
            command.arg(format!("-{group}"));
        }
    }
    // Include the shell even if ps raced with its startup/exit or job control.
    command.arg(id.to_string());
    let output = command.output().context("terminate terminal processes")?;
    ensure!(
        output.status.success() || exited.load(Ordering::Acquire),
        "terminal process stop failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}
