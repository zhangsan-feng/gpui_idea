use alacritty_terminal::tty::Pty;
use anyhow::{Context, Result, ensure};
use std::{
    fs::File,
    os::fd::AsRawFd,
    sync::atomic::{AtomicBool, Ordering},
};

pub(in crate::infrastructure::terminal) struct ProcessJob {
    group: libc::pid_t,
    terminal: File,
    stopped: AtomicBool,
}

impl ProcessJob {
    pub(in crate::infrastructure::terminal) fn new(pty: &Pty) -> Result<Self> {
        // Alacritty calls setsid before exec, so the shell PID is also its PGID.
        let group = libc::pid_t::try_from(pty.child().id())?;
        ensure!(group > 1, "invalid terminal process group");
        Ok(Self {
            group,
            terminal: pty
                .file()
                .try_clone()
                .context("retain terminal control handle")?,
            stopped: AtomicBool::new(false),
        })
    }

    pub(in crate::infrastructure::terminal) fn terminate(&self) -> Result<()> {
        if self.stopped.load(Ordering::Acquire) {
            return Ok(());
        }
        // Interactive job control also creates background/stopped groups. Discover
        // all groups in this PTY's session, using the common macOS/Linux ps interface.
        let output = std::process::Command::new("/bin/ps")
            .args(["-axo", "pid="])
            .output()
            .context("enumerate terminal session processes")?;
        ensure!(
            output.status.success(),
            "cannot enumerate terminal session processes"
        );
        let mut groups = std::collections::BTreeSet::new();
        for pid in String::from_utf8_lossy(&output.stdout).split_whitespace() {
            let Ok(pid) = pid.parse::<libc::pid_t>() else {
                continue;
            };
            if pid > 1 && unsafe { libc::getsid(pid) } == self.group {
                let group = unsafe { libc::getpgid(pid) };
                if group > 1 {
                    groups.insert(group);
                }
            }
        }
        // A command may have started while ps was running; include the foreground.
        let foreground = unsafe { libc::tcgetpgrp(self.terminal.as_raw_fd()) };
        if foreground > 1 && foreground != self.group {
            // Check the session too; never signal a group belonging to another PTY.
            if unsafe { libc::getsid(foreground) } == self.group {
                groups.insert(foreground);
            }
        }
        // Kill job groups before the shell so they cannot be orphaned by its exit.
        let has_shell_group = groups.remove(&self.group);
        for group in groups {
            signal_group(group, libc::SIGKILL)?;
        }
        if has_shell_group {
            signal_group(self.group, libc::SIGKILL)?;
        }
        self.stopped.store(true, Ordering::Release);
        Ok(())
    }
}

fn signal_group(group: libc::pid_t, signal: libc::c_int) -> Result<()> {
    if unsafe { libc::kill(-group, signal) } == -1 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::ESRCH) {
            return Err(error).context("terminate terminal process group");
        }
    }
    Ok(())
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        if let Err(error) = self.terminate() {
            tracing::warn!(%error, "terminal process group cleanup failed");
        }
    }
}
