use crate::domain::settings::GeneralSettings;
use anyhow::{Context as _, Result};
use std::fs::{File, OpenOptions};

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

fn lock_settings() -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path().with_extension("lock"))
        .context("无法创建设置文件锁")?;
    file.lock().context("无法锁定设置文件")?;
    Ok(file)
}

pub(super) fn save(
    mut value: GeneralSettings,
    preserve_editor_size: bool,
) -> Result<GeneralSettings> {
    let _lock = lock_settings()?;
    if preserve_editor_size {
        value.appearance.editor_size = load()?.appearance.editor_size;
    }
    write(&value)?;
    Ok(value)
}

pub(super) fn save_editor_font_size(size: u16) -> Result<()> {
    let _lock = lock_settings()?;
    let mut value = load()?;
    if value.appearance.editor_size != size {
        value.appearance.editor_size = size;
        write(&value)?;
    }
    Ok(())
}

fn write(value: &GeneralSettings) -> Result<()> {
    let path = path();
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result: Result<()> = (|| {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(value)?)?;
        file.sync_all()?;
        std::fs::rename(&temporary, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.context("无法保存设置")
}
