use anyhow::{Context, Result};
use std::{borrow::Cow, collections::BTreeMap, path::Path, sync::OnceLock};

static ASSETS: OnceLock<BTreeMap<String, Vec<u8>>> = OnceLock::new();

// Preload once before GPUI starts; rendering never reads external files.
pub(crate) fn initialize(directory: &Path) -> Result<()> {
    let mut assets = BTreeMap::new();
    let mut directories = vec![directory.to_owned()];
    let mut total = 0;
    while let Some(current) = directories.pop() {
        for entry in std::fs::read_dir(&current)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                directories.push(entry.path());
                continue;
            }
            if !kind.is_file() {
                continue;
            }
            let size = entry.metadata()?.len();
            total += size;
            anyhow::ensure!(
                size <= 8 * 1024 * 1024 && total <= 32 * 1024 * 1024,
                "外部 assets 超过加载上限（单文件 8 MiB，总计 32 MiB）"
            );
            let key = entry
                .path()
                .strip_prefix(directory)?
                .to_string_lossy()
                .replace('\\', "/");
            let bytes =
                std::fs::read(entry.path()).with_context(|| format!("无法读取资源 {key}"))?;
            assets.insert(key, bytes);
        }
    }
    tracing::info!(path = %directory.display(), count = assets.len(), "external assets preloaded");
    let _ = ASSETS.set(assets);
    Ok(())
}

pub(crate) fn load(path: &str) -> Option<Cow<'static, [u8]>> {
    ASSETS
        .get()?
        .get(path)
        .map(|bytes| Cow::Borrowed(bytes.as_slice()))
}

pub(crate) fn list(prefix: &str) -> impl Iterator<Item = &'static str> {
    ASSETS
        .get()
        .into_iter()
        .flat_map(|assets| assets.keys())
        .map(String::as_str)
        .filter(move |path| path.starts_with(prefix))
}
