use anyhow::{Context as _, Result};
use windows::{
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_ITEMS, ERROR_PATH_NOT_FOUND},
        System::Registry::*,
    },
    core::{PCWSTR, PWSTR},
};

pub(super) struct Key(HKEY);

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

impl Drop for Key {
    fn drop(&mut self) {
        // This handle is owned by Key and is never a predefined root handle.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

impl Key {
    pub(super) fn value_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        // Windows registry value names are limited to 16,383 UTF-16 characters.
        let mut buffer = vec![0u16; 16_384];
        for index in 0.. {
            let mut length = buffer.len() as u32;
            let result = unsafe {
                RegEnumValueW(
                    self.0,
                    index,
                    Some(PWSTR(buffer.as_mut_ptr())),
                    &mut length,
                    None,
                    None,
                    None,
                    None,
                )
            };
            if result == ERROR_NO_MORE_ITEMS {
                break;
            }
            result.ok().context("无法枚举已注册的文件类型")?;
            names.push(
                String::from_utf16(&buffer[..length as usize])
                    .context("文件类型名称不是有效的 Unicode")?,
            );
        }
        Ok(names)
    }

    pub(super) fn open(path: &str) -> Result<Option<Self>> {
        let path = wide(path);
        let mut handle = HKEY::default();
        // The UTF-16 path remains alive for the duration of the call.
        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                KEY_READ,
                &mut handle,
            )
        };
        if result == ERROR_FILE_NOT_FOUND || result == ERROR_PATH_NOT_FOUND {
            return Ok(None);
        }
        result.ok().context("无法读取文件关联注册表")?;
        Ok(Some(Self(handle)))
    }

    pub(super) fn create(path: &str) -> Result<Self> {
        let path = wide(path);
        let mut handle = HKEY::default();
        // No security descriptor is passed; the key inherits the current user's ACL.
        unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                None,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_READ | KEY_WRITE,
                None,
                &mut handle,
                None,
            )
            .ok()
            .context("无法创建文件关联注册表项")?;
        }
        Ok(Self(handle))
    }

    pub(super) fn set(&self, name: &str, value: &str) -> Result<()> {
        let name = wide(name);
        let bytes = wide(value)
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        // REG_SZ includes its terminating NUL and the byte slice is valid during the call.
        unsafe {
            RegSetValueExW(self.0, PCWSTR(name.as_ptr()), None, REG_SZ, Some(&bytes))
                .ok()
                .context("无法写入文件关联注册表值")?;
        }
        Ok(())
    }

    pub(super) fn get(&self, name: &str) -> Result<Option<String>> {
        let name = wide(name);
        let mut size = 0;
        // Query the required byte count before allocating a UTF-16 buffer.
        let result = unsafe {
            RegGetValueW(
                self.0,
                PCWSTR::null(),
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            )
        };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        result.ok().context("无法读取文件关联注册表值")?;
        let mut buffer = vec![0u16; (size as usize).div_ceil(2)];
        unsafe {
            RegGetValueW(
                self.0,
                PCWSTR::null(),
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut size),
            )
            .ok()
            .context("无法读取文件关联注册表值")?;
        }
        let length = buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len());
        Ok(Some(
            String::from_utf16(&buffer[..length]).context("文件关联值不是有效的 Unicode")?,
        ))
    }
}
