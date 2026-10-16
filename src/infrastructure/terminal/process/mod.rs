#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub(in crate::infrastructure::terminal) use unix::ProcessJob;
#[cfg(windows)]
pub(in crate::infrastructure::terminal) use windows::ProcessJob;
