use anyhow::{Context, Result};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject,
    },
};

// The job owns only this PTY's process tree, including GUI children launched by cargo.
pub(in crate::infrastructure::terminal) struct ProcessJob(usize);
impl ProcessJob {
    pub(in crate::infrastructure::terminal) fn new(
        pty: &alacritty_terminal::tty::Pty,
    ) -> Result<Self> {
        let child = pty.child_watcher().raw_handle();
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error()).context("create terminal process job");
        }
        let job = Self(handle as usize);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let success = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const _,
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if success == 0 {
            return Err(std::io::Error::last_os_error()).context("configure terminal process job");
        }
        if unsafe { AssignProcessToJobObject(handle, child) } == 0 {
            return Err(std::io::Error::last_os_error())
                .context("assign terminal child process to job");
        }
        Ok(job)
    }
    pub(in crate::infrastructure::terminal) fn terminate(&self) -> Result<()> {
        if unsafe { TerminateJobObject(self.0 as HANDLE, 130) } == 0 {
            return Err(std::io::Error::last_os_error()).context("terminate terminal process tree");
        }
        Ok(())
    }
}
impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0 as HANDLE);
        }
    }
}
