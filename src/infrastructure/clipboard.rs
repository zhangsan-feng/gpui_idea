use windows::Win32::{
    Foundation::HGLOBAL,
    System::{
        DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard},
        Memory::{GlobalLock, GlobalSize, GlobalUnlock},
        Ole::CF_UNICODETEXT,
    },
};

pub(crate) fn read_text() -> Option<String> {
    // Windows owns the clipboard handle. Keep both the clipboard and the
    // memory locked while copying the null-terminated UTF-16 text.
    unsafe { OpenClipboard(None).ok()? };
    let _clipboard = Clipboard;
    let handle = HGLOBAL(unsafe { GetClipboardData(u32::from(CF_UNICODETEXT.0)).ok()? }.0);
    let units = unsafe { GlobalSize(handle) } / size_of::<u16>();
    let pointer = unsafe { GlobalLock(handle) }.cast::<u16>();
    if pointer.is_null() {
        return None;
    }
    let _lock = MemoryLock(handle);
    let buffer = unsafe { std::slice::from_raw_parts(pointer, units) };
    let len = buffer.iter().position(|&unit| unit == 0)?;
    Some(String::from_utf16_lossy(&buffer[..len]))
}

struct Clipboard;

impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

struct MemoryLock(HGLOBAL);

impl Drop for MemoryLock {
    fn drop(&mut self) {
        let _ = unsafe { GlobalUnlock(self.0) };
    }
}
