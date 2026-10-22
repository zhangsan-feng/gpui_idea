use anyhow::Result;

pub(super) fn require_online() -> Result<()> {
    anyhow::ensure!(
        is_online(),
        "当前无可用网络连接，已跳过语言服务下载；请联网后在设置中重试安装"
    );
    Ok(())
}

// Read existing OS connectivity state; do not send an application network probe.
#[cfg(windows)]
pub(super) fn is_online() -> bool {
    match connectivity() {
        Ok(online) => {
            tracing::debug!(online, "LSP network connectivity checked");
            online
        }
        Err(error) => {
            tracing::warn!(%error, "Cannot determine connectivity; skipping LSP download");
            false
        }
    }
}

#[cfg(windows)]
fn connectivity() -> windows::core::Result<bool> {
    use windows::Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        Networking::NetworkListManager::{INetworkListManager, NetworkListManager},
        System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx},
    };

    // SAFETY: initialize COM on this thread, without changing an existing apartment.
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if initialized != RPC_E_CHANGED_MODE {
        initialized.ok()?;
    }
    let _apartment = ComApartment(initialized.is_ok());
    // SAFETY: COM is initialized; the interface stays on this thread and is
    // released before the apartment guard. NLM is available on older Windows too.
    let manager: INetworkListManager =
        unsafe { CoCreateInstance(&NetworkListManager, None, CLSCTX_ALL)? };
    unsafe { manager.IsConnectedToInternet().map(|value| value.as_bool()) }
}

#[cfg(windows)]
struct ComApartment(bool);

#[cfg(windows)]
impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: balance this thread's successful CoInitializeEx call,
            // including S_FALSE, without uninitializing an existing STA.
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

// Native managed downloads currently support Windows only. Other platforms
// retain their existing npm/Go behavior until a connectivity provider is added.
#[cfg(not(windows))]
pub(super) fn is_online() -> bool {
    true
}
