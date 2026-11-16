use super::SettingsService;
use anyhow::{Result, ensure};

impl SettingsService {
    pub(super) fn parse_ignored_directories_core(&self, input: &str) -> Vec<String> {
        input
            .split([';', '；', ',', '，', '\n', '\r'])
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect()
    }

    pub(super) fn normalize_ignored_directories(&self, names: &mut Vec<String>) -> Result<()> {
        let mut normalized = Vec::<String>::new();
        for name in names.iter() {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            ensure!(
                name != "."
                    && name != ".."
                    && !name.ends_with(['.', ' '])
                    && !name
                        .chars()
                        .any(|ch| ch.is_control() || "<>:\"/\\|?*".contains(ch)),
                "忽略目录请输入目录名称，不支持路径或通配符：{name}"
            );
            if !normalized.iter().any(|existing| {
                if cfg!(windows) {
                    existing.eq_ignore_ascii_case(name)
                } else {
                    existing == name
                }
            }) {
                normalized.push(name.to_owned());
            }
        }
        *names = normalized;
        Ok(())
    }
}
