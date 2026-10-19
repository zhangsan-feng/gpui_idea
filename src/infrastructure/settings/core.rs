use crate::domain::settings::GeneralSettings;
use anyhow::{Context as _, Result};

fn path() -> std::path::PathBuf {
    super::super::paths::get().data.join("settings.json")
}

pub(super) fn load() -> Result<GeneralSettings> {
    match std::fs::read(path()) {
        Ok(bytes) => serde_json::from_slice(&bytes).context("设置文件格式无效"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Default::default()),
        Err(error) => Err(error).context("无法读取设置"),
    }
}

pub(super) fn save(value: GeneralSettings) -> Result<()> {
    let path = path();
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result: Result<()> = (|| {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(&value)?)?;
        file.sync_all()?;
        std::fs::rename(&temporary, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.context("无法保存设置")
}
