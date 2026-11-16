use anyhow::{Context as _, Result, ensure};
use windows::{
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            AL_EFFECTIVE, AT_FILEEXTENSION, ApplicationAssociationRegistration,
            IApplicationAssociationRegistration,
        },
    },
    core::PCWSTR,
};

struct Apartment(bool);

impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            // Balance only our successful COM initialization, on this same blocking worker.
            unsafe { CoUninitialize() };
        }
    }
}

pub(super) fn query(extensions: &[String], prog_id: &str) -> Result<Vec<String>> {
    let initialized = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    if initialized != RPC_E_CHANGED_MODE {
        initialized.ok().context("无法初始化默认应用查询")?;
    }
    let _apartment = Apartment(initialized.is_ok());
    // Query the effective association instead of interpreting UserChoice registry internals.
    let registration: IApplicationAssociationRegistration = unsafe {
        CoCreateInstance(
            &ApplicationAssociationRegistration,
            None,
            CLSCTX_INPROC_SERVER,
        )
    }
    .context("无法创建 Windows 默认应用查询接口")?;
    let mut defaults = Vec::new();
    for extension in extensions {
        let wide = extension.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
        let result = unsafe {
            registration.QueryCurrentDefault(PCWSTR(wide.as_ptr()), AT_FILEEXTENSION, AL_EFFECTIVE)
        };
        match result {
            Ok(pointer) => {
                ensure!(!pointer.is_null(), "Windows 返回了空的默认应用标识");
                // The returned ProgID is allocated by COM; free it even if decoding fails.
                let current = unsafe { pointer.to_string() };
                unsafe { CoTaskMemFree(Some(pointer.0.cast())) };
                let current = current.context("默认应用标识不是有效的 Unicode")?;
                let is_default = current.eq_ignore_ascii_case(prog_id);
                tracing::debug!(%extension, default_prog_id = %current, is_default, "effective file association queried");
                if is_default {
                    defaults.push(extension.clone());
                }
            }
            Err(error) => {
                // Unassigned types and failed queries must never appear as confirmed defaults.
                tracing::debug!(%extension, %error, "default file association could not be confirmed");
            }
        }
    }
    tracing::info!(default_extensions = ?defaults, checked = extensions.len(), "Windows default file associations refreshed");
    Ok(defaults)
}
